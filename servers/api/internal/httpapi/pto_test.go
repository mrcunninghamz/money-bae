package httpapi

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func newPtoTestRouter(t *testing.T) (http.Handler, *gorm.DB) {
	db := testdb.New(t, &models.User{}, &models.Pto{}, &models.PtoPlan{}, &models.HolidayHour{})
	router := NewRouter(db, auth.MockVerifier{DB: db})
	return router, db
}

func decodePtoResponse(t *testing.T, rec *httptest.ResponseRecorder) ptoResponse {
	t.Helper()
	var got ptoResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	return got
}

func TestCreatePto_MinimalFields_DefaultsHoursAndComputesRemaining(t *testing.T) {
	router, _ := newPtoTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/ptos", ptoRequest{
		Year:           2027,
		AvailableHours: decimal.NewFromInt(200),
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodePtoResponse(t, rec)
	if !got.HoursUsed.IsZero() || !got.HoursPlanned.IsZero() {
		t.Fatalf("expected zero-defaulted hoursUsed/hoursPlanned, got %+v", got)
	}
	if !got.HoursRemaining.Equal(decimal.NewFromInt(200)) {
		t.Fatalf("expected hoursRemaining 200, got %s", got.HoursRemaining)
	}
}

func TestCreatePto_MissingYear_Returns400(t *testing.T) {
	router, _ := newPtoTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/ptos", ptoRequest{
		AvailableHours: decimal.NewFromInt(200),
	})

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestListPtos_OrderParam_DefaultsNewestFirst(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	older := models.Pto{UserID: user.ID, Year: 2024, AvailableHours: decimal.NewFromInt(184), HoursRemaining: decimal.NewFromInt(184)}
	if err := db.Create(&older).Error; err != nil {
		t.Fatalf("failed to seed older pto: %v", err)
	}
	newer := models.Pto{UserID: user.ID, Year: 2026, AvailableHours: decimal.NewFromInt(200), HoursRemaining: decimal.NewFromInt(200)}
	if err := db.Create(&newer).Error; err != nil {
		t.Fatalf("failed to seed newer pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ptos", nil)
	var got []ptoResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != newer.ID || got[1].ID != older.ID {
		t.Fatalf("expected [newer, older] by default, got %+v", got)
	}

	rec = doJSON(t, router, http.MethodGet, "/ptos?order=asc", nil)
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != older.ID || got[1].ID != newer.ID {
		t.Fatalf("expected [older, newer] with order=asc, got %+v", got)
	}
}

func TestGetPto_IncludesPtoPlansAndHolidayHours(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200), HoursRemaining: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}
	plan := models.PtoPlan{
		PtoID: pto.ID, StartDate: time.Now(), EndDate: time.Now(),
		Name: "Christmas", Hours: decimal.NewFromInt(72), Status: "Planned",
	}
	if err := db.Create(&plan).Error; err != nil {
		t.Fatalf("failed to seed pto plan: %v", err)
	}
	holiday := models.HolidayHour{PtoID: pto.ID, Date: time.Now(), Name: "Christmas Day", Hours: decimal.NewFromInt(8)}
	if err := db.Create(&holiday).Error; err != nil {
		t.Fatalf("failed to seed holiday hour: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ptos/"+pto.ID.String(), nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got ptoDetailResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if len(got.PtoPlans) != 1 || got.PtoPlans[0].ID != plan.ID {
		t.Fatalf("expected 1 nested pto plan, got %+v", got.PtoPlans)
	}
	if len(got.HolidayHours) != 1 || got.HolidayHours[0].ID != holiday.ID {
		t.Fatalf("expected 1 nested holiday hour, got %+v", got.HolidayHours)
	}
}

func TestGetPto_BelongsToAnotherUser_Returns404(t *testing.T) {
	router, db := newPtoTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	pto := models.Pto{UserID: otherUser.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ptos/"+pto.ID.String(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeletePto_CascadesPlansAndHolidays(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}
	plan := models.PtoPlan{
		PtoID: pto.ID, StartDate: time.Now(), EndDate: time.Now(),
		Name: "Christmas", Hours: decimal.NewFromInt(72), Status: "Planned",
	}
	if err := db.Create(&plan).Error; err != nil {
		t.Fatalf("failed to seed pto plan: %v", err)
	}
	holiday := models.HolidayHour{PtoID: pto.ID, Date: time.Now(), Name: "Christmas Day", Hours: decimal.NewFromInt(8)}
	if err := db.Create(&holiday).Error; err != nil {
		t.Fatalf("failed to seed holiday hour: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ptos/"+pto.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var planCount, holidayCount int64
	db.Model(&models.PtoPlan{}).Where("pto_id = ?", pto.ID).Count(&planCount)
	db.Model(&models.HolidayHour{}).Where("pto_id = ?", pto.ID).Count(&holidayCount)
	if planCount != 0 || holidayCount != 0 {
		t.Fatalf("expected plans and holidays to be cascade-deleted, got %d plans, %d holidays", planCount, holidayCount)
	}
}

// --- PtoPlan (nested)

func TestCreatePtoPlan_RecomputesParentHours(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200), HoursRemaining: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ptos/"+pto.ID.String()+"/plans", ptoPlanRequest{
		StartDate: time.Date(2025, 12, 16, 0, 0, 0, 0, time.UTC),
		EndDate:   time.Date(2025, 12, 31, 0, 0, 0, 0, time.UTC),
		Name:      "Christmas",
		Hours:     decimal.NewFromInt(72),
		Status:    "Completed",
	})
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var reloaded models.Pto
	if err := db.First(&reloaded, "id = ?", pto.ID).Error; err != nil {
		t.Fatalf("failed to reload pto: %v", err)
	}
	if !reloaded.HoursUsed.Equal(decimal.NewFromInt(72)) {
		t.Fatalf("expected hoursUsed 72, got %s", reloaded.HoursUsed)
	}
	if !reloaded.HoursRemaining.Equal(decimal.NewFromInt(128)) {
		t.Fatalf("expected hoursRemaining 128 (200-72), got %s", reloaded.HoursRemaining)
	}
}

func TestCreatePtoPlan_InvalidStatus_Returns400(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ptos/"+pto.ID.String()+"/plans", ptoPlanRequest{
		StartDate: time.Now(), EndDate: time.Now(), Name: "Christmas",
		Hours: decimal.NewFromInt(72), Status: "Bogus",
	})
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCreatePtoPlan_PtoNotOwnedByUser_Returns404(t *testing.T) {
	router, db := newPtoTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	pto := models.Pto{UserID: otherUser.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ptos/"+pto.ID.String()+"/plans", ptoPlanRequest{
		StartDate: time.Now(), EndDate: time.Now(), Name: "Christmas",
		Hours: decimal.NewFromInt(72), Status: "Planned",
	})
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestUpdatePtoPlan_StatusChangeMovesHoursBetweenPlannedAndUsed(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200), HoursRemaining: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}
	plan := models.PtoPlan{
		PtoID: pto.ID, StartDate: time.Now(), EndDate: time.Now(),
		Name: "Christmas", Hours: decimal.NewFromInt(72), Status: "Planned",
	}
	if err := db.Create(&plan).Error; err != nil {
		t.Fatalf("failed to seed pto plan: %v", err)
	}
	if err := recomputePtoHours(db, pto.ID); err != nil {
		t.Fatalf("failed to seed initial recompute: %v", err)
	}

	rec := doJSON(t, router, http.MethodPut, "/ptos/"+pto.ID.String()+"/plans/"+plan.ID.String(), ptoPlanRequest{
		StartDate: plan.StartDate, EndDate: plan.EndDate, Name: plan.Name,
		Hours: plan.Hours, Status: "Completed",
	})
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var reloaded models.Pto
	if err := db.First(&reloaded, "id = ?", pto.ID).Error; err != nil {
		t.Fatalf("failed to reload pto: %v", err)
	}
	if !reloaded.HoursPlanned.IsZero() {
		t.Fatalf("expected hoursPlanned 0 after status change, got %s", reloaded.HoursPlanned)
	}
	if !reloaded.HoursUsed.Equal(decimal.NewFromInt(72)) {
		t.Fatalf("expected hoursUsed 72 after status change, got %s", reloaded.HoursUsed)
	}
}

func TestDeletePtoPlan_RecomputesParentHours(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200), HoursRemaining: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}
	plan := models.PtoPlan{
		PtoID: pto.ID, StartDate: time.Now(), EndDate: time.Now(),
		Name: "Christmas", Hours: decimal.NewFromInt(72), Status: "Completed",
	}
	if err := db.Create(&plan).Error; err != nil {
		t.Fatalf("failed to seed pto plan: %v", err)
	}
	if err := recomputePtoHours(db, pto.ID); err != nil {
		t.Fatalf("failed to seed initial recompute: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ptos/"+pto.ID.String()+"/plans/"+plan.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var reloaded models.Pto
	if err := db.First(&reloaded, "id = ?", pto.ID).Error; err != nil {
		t.Fatalf("failed to reload pto: %v", err)
	}
	if !reloaded.HoursUsed.IsZero() {
		t.Fatalf("expected hoursUsed 0 after delete, got %s", reloaded.HoursUsed)
	}
	if !reloaded.HoursRemaining.Equal(decimal.NewFromInt(200)) {
		t.Fatalf("expected hoursRemaining back to 200, got %s", reloaded.HoursRemaining)
	}
}

func TestDeletePtoPlan_NotFound_Returns404(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ptos/"+pto.ID.String()+"/plans/"+uuid.NewString(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

// --- HolidayHour (nested)

func TestCreateHolidayHour_Succeeds(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ptos/"+pto.ID.String()+"/holidays", holidayHourRequest{
		Date: time.Date(2025, 12, 25, 0, 0, 0, 0, time.UTC), Name: "Christmas Day", Hours: decimal.NewFromInt(8),
	})
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var got holidayHourResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if got.PtoID != pto.ID || got.Name != "Christmas Day" {
		t.Fatalf("expected ptoId %s and name Christmas Day, got %+v", pto.ID, got)
	}

	// Holiday hours are informational only — must not affect the parent's
	// hoursRemaining (matches existing app behavior).
	var reloaded models.Pto
	if err := db.First(&reloaded, "id = ?", pto.ID).Error; err != nil {
		t.Fatalf("failed to reload pto: %v", err)
	}
	if !reloaded.HoursRemaining.IsZero() {
		t.Fatalf("expected hoursRemaining untouched by holiday hours, got %s", reloaded.HoursRemaining)
	}
}

func TestDeleteHolidayHour_Succeeds(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}
	holiday := models.HolidayHour{PtoID: pto.ID, Date: time.Now(), Name: "Christmas Day", Hours: decimal.NewFromInt(8)}
	if err := db.Create(&holiday).Error; err != nil {
		t.Fatalf("failed to seed holiday hour: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ptos/"+pto.ID.String()+"/holidays/"+holiday.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var count int64
	db.Model(&models.HolidayHour{}).Where("id = ?", holiday.ID).Count(&count)
	if count != 0 {
		t.Fatalf("expected holiday hour to be deleted, found %d rows", count)
	}
}

func TestDeleteHolidayHour_NotFound_Returns404(t *testing.T) {
	router, db := newPtoTestRouter(t)
	user := seedCurrentUser(t, db)

	pto := models.Pto{UserID: user.ID, Year: 2025, AvailableHours: decimal.NewFromInt(200)}
	if err := db.Create(&pto).Error; err != nil {
		t.Fatalf("failed to seed pto: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ptos/"+pto.ID.String()+"/holidays/"+uuid.NewString(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}
