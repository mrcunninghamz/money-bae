package httpapi

import (
	"encoding/json"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type ptoRequest struct {
	Year           int             `json:"year"`
	PrevYearHours  decimal.Decimal `json:"prevYearHours"`
	AvailableHours decimal.Decimal `json:"availableHours"`
	RolloverHours  bool            `json:"rolloverHours"`
}

type ptoResponse struct {
	ID             uuid.UUID       `json:"id"`
	Year           int             `json:"year"`
	PrevYearHours  decimal.Decimal `json:"prevYearHours"`
	AvailableHours decimal.Decimal `json:"availableHours"`
	HoursPlanned   decimal.Decimal `json:"hoursPlanned"`
	HoursUsed      decimal.Decimal `json:"hoursUsed"`
	HoursRemaining decimal.Decimal `json:"hoursRemaining"`
	RolloverHours  bool            `json:"rolloverHours"`
	CreatedAt      time.Time       `json:"createdAt"`
	UpdatedAt      time.Time       `json:"updatedAt"`
}

func toPtoResponse(pto models.Pto) ptoResponse {
	return ptoResponse{
		ID:             pto.ID,
		Year:           pto.Year,
		PrevYearHours:  pto.PrevYearHours,
		AvailableHours: pto.AvailableHours,
		HoursPlanned:   pto.HoursPlanned,
		HoursUsed:      pto.HoursUsed,
		HoursRemaining: pto.HoursRemaining,
		RolloverHours:  pto.RolloverHours,
		CreatedAt:      pto.CreatedAt,
		UpdatedAt:      pto.UpdatedAt,
	}
}

// ptoDetailResponse is the "detail" shape (GET /ptos/{id} only): the pto
// year's own fields plus its navigational properties — every plan entry and
// holiday hour recorded against it. List/create/update stay lean.
type ptoDetailResponse struct {
	ptoResponse
	PtoPlans     []ptoPlanResponse     `json:"ptoPlans"`
	HolidayHours []holidayHourResponse `json:"holidayHours"`
}

func toPtoDetailResponse(pto models.Pto) ptoDetailResponse {
	plans := make([]ptoPlanResponse, len(pto.PtoPlans))
	for i, p := range pto.PtoPlans {
		plans[i] = toPtoPlanResponse(p)
	}
	holidays := make([]holidayHourResponse, len(pto.HolidayHours))
	for i, h := range pto.HolidayHours {
		holidays[i] = toHolidayHourResponse(h)
	}
	return ptoDetailResponse{
		ptoResponse:  toPtoResponse(pto),
		PtoPlans:     plans,
		HolidayHours: holidays,
	}
}

func decodePtoRequest(w http.ResponseWriter, r *http.Request) (ptoRequest, bool) {
	var req ptoRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return ptoRequest{}, false
	}
	if req.Year == 0 {
		http.Error(w, "year is required", http.StatusBadRequest)
		return ptoRequest{}, false
	}
	return req, true
}

// ptoOwnedByUser reports whether a pto year record with the given id exists
// and belongs to userID — checked before attaching a plan or holiday hour.
func ptoOwnedByUser(db *gorm.DB, ptoID, userID uuid.UUID) bool {
	var count int64
	db.Model(&models.Pto{}).Where("id = ? AND user_id = ?", ptoID, userID).Count(&count)
	return count > 0
}

func findOwnedPto(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID, pathParam string) (models.Pto, bool) {
	var pto models.Pto
	id, err := uuid.Parse(r.PathValue(pathParam))
	if err != nil {
		http.Error(w, "invalid id", http.StatusBadRequest)
		return pto, false
	}
	err = db.Where("id = ? AND user_id = ?", id, userID).First(&pto).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		http.Error(w, "pto not found", http.StatusNotFound)
		return pto, false
	}
	if err != nil {
		http.Error(w, "failed to look up pto", http.StatusInternalServerError)
		return pto, false
	}
	return pto, true
}

// currentPtoHandler backs the home dashboard's PTO shortcut, which only
// needs to know whether the current calendar year has a PTO record and its
// remaining hours — not the full PTO history.
func currentPtoHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		var pto models.Pto
		err := db.Where("user_id = ? AND year = ?", principal.UserID, time.Now().Year()).First(&pto).Error
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "no pto for the current year", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to look up current pto", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toPtoResponse(pto))
	}
}

func createPtoHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		req, ok := decodePtoRequest(w, r)
		if !ok {
			return
		}

		pto := models.Pto{
			UserID:         principal.UserID,
			Year:           req.Year,
			PrevYearHours:  req.PrevYearHours,
			AvailableHours: req.AvailableHours,
			HoursPlanned:   decimal.Zero,
			HoursUsed:      decimal.Zero,
			HoursRemaining: req.AvailableHours.Add(req.PrevYearHours),
			RolloverHours:  req.RolloverHours,
		}

		if err := db.Create(&pto).Error; err != nil {
			http.Error(w, "failed to create pto", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, toPtoResponse(pto))
	}
}

func listPtosHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		var ptos []models.Pto
		order := "year " + orderDirection(r)
		if err := db.Where("user_id = ?", principal.UserID).Order(order).Find(&ptos).Error; err != nil {
			http.Error(w, "failed to list ptos", http.StatusInternalServerError)
			return
		}

		responses := make([]ptoResponse, len(ptos))
		for i, pto := range ptos {
			responses[i] = toPtoResponse(pto)
		}

		writeJSON(w, http.StatusOK, responses)
	}
}

func getPtoHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		id, err := uuid.Parse(r.PathValue("id"))
		if err != nil {
			http.Error(w, "invalid id", http.StatusBadRequest)
			return
		}

		var pto models.Pto
		err = db.Preload("PtoPlans").Preload("HolidayHours").
			Where("id = ? AND user_id = ?", id, principal.UserID).First(&pto).Error
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "pto not found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to look up pto", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toPtoDetailResponse(pto))
	}
}

func updatePtoHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		pto, ok := findOwnedPto(w, r, db, principal.UserID, "id")
		if !ok {
			return
		}

		req, ok := decodePtoRequest(w, r)
		if !ok {
			return
		}

		pto.Year = req.Year
		pto.PrevYearHours = req.PrevYearHours
		pto.AvailableHours = req.AvailableHours
		pto.RolloverHours = req.RolloverHours
		pto.HoursRemaining = req.AvailableHours.Add(req.PrevYearHours).Sub(pto.HoursUsed).Sub(pto.HoursPlanned)

		if err := db.Save(&pto).Error; err != nil {
			http.Error(w, "failed to update pto", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toPtoResponse(pto))
	}
}

func deletePtoHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		id, err := uuid.Parse(r.PathValue("id"))
		if err != nil {
			http.Error(w, "invalid id", http.StatusBadRequest)
			return
		}

		err = db.Transaction(func(tx *gorm.DB) error {
			var pto models.Pto
			if err := tx.Where("id = ? AND user_id = ?", id, principal.UserID).First(&pto).Error; err != nil {
				return err
			}
			if err := tx.Where("pto_id = ?", id).Delete(&models.PtoPlan{}).Error; err != nil {
				return err
			}
			if err := tx.Where("pto_id = ?", id).Delete(&models.HolidayHour{}).Error; err != nil {
				return err
			}
			return tx.Delete(&pto).Error
		})
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "pto not found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to delete pto", http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}

// --- PtoPlan: an individual PTO entry (nested under a Pto year). No
// standalone GET; reads happen via GET /ptos/{id}'s embedded ptoPlans.

type ptoPlanRequest struct {
	StartDate   time.Time       `json:"startDate"`
	EndDate     time.Time       `json:"endDate"`
	Name        string          `json:"name"`
	Description *string         `json:"description"`
	Hours       decimal.Decimal `json:"hours"`
	Status      string          `json:"status"`
	CustomHours bool            `json:"customHours"`
}

type ptoPlanResponse struct {
	ID          uuid.UUID       `json:"id"`
	PtoID       uuid.UUID       `json:"ptoId"`
	StartDate   time.Time       `json:"startDate"`
	EndDate     time.Time       `json:"endDate"`
	Name        string          `json:"name"`
	Description *string         `json:"description"`
	Hours       decimal.Decimal `json:"hours"`
	Status      string          `json:"status"`
	CustomHours bool            `json:"customHours"`
	CreatedAt   time.Time       `json:"createdAt"`
	UpdatedAt   time.Time       `json:"updatedAt"`
}

func toPtoPlanResponse(p models.PtoPlan) ptoPlanResponse {
	return ptoPlanResponse{
		ID:          p.ID,
		PtoID:       p.PtoID,
		StartDate:   p.StartDate,
		EndDate:     p.EndDate,
		Name:        p.Name,
		Description: p.Description,
		Hours:       p.Hours,
		Status:      p.Status,
		CustomHours: p.CustomHours,
		CreatedAt:   p.CreatedAt,
		UpdatedAt:   p.UpdatedAt,
	}
}

var validPtoPlanStatuses = map[string]bool{"Planned": true, "Completed": true}

func decodePtoPlanRequest(w http.ResponseWriter, r *http.Request) (models.PtoPlan, bool) {
	var req ptoPlanRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return models.PtoPlan{}, false
	}
	if req.Name == "" {
		http.Error(w, "name is required", http.StatusBadRequest)
		return models.PtoPlan{}, false
	}
	if req.StartDate.IsZero() || req.EndDate.IsZero() {
		http.Error(w, "startDate and endDate are required", http.StatusBadRequest)
		return models.PtoPlan{}, false
	}
	if !validPtoPlanStatuses[req.Status] {
		http.Error(w, `status must be "Planned" or "Completed"`, http.StatusBadRequest)
		return models.PtoPlan{}, false
	}
	return models.PtoPlan{
		StartDate:   req.StartDate,
		EndDate:     req.EndDate,
		Name:        req.Name,
		Description: req.Description,
		Hours:       req.Hours,
		Status:      req.Status,
		CustomHours: req.CustomHours,
	}, true
}

// recomputePtoHours re-sums this pto's plans into HoursPlanned ("Planned"
// status) / HoursUsed ("Completed" status) and recomputes HoursRemaining,
// persisting the parent Pto — called after every PtoPlan create/update/
// delete so the balance card always reflects real entries, matching the
// app's existing "hours used = sum of entries" behavior. Both planned and
// used hours are treated as already spoken-for when computing remaining.
func recomputePtoHours(tx *gorm.DB, ptoID uuid.UUID) error {
	var plans []models.PtoPlan
	if err := tx.Where("pto_id = ?", ptoID).Find(&plans).Error; err != nil {
		return err
	}
	planned, used := decimal.Zero, decimal.Zero
	for _, p := range plans {
		if p.Status == "Completed" {
			used = used.Add(p.Hours)
		} else {
			planned = planned.Add(p.Hours)
		}
	}

	var pto models.Pto
	if err := tx.Where("id = ?", ptoID).First(&pto).Error; err != nil {
		return err
	}
	pto.HoursPlanned = planned
	pto.HoursUsed = used
	pto.HoursRemaining = pto.AvailableHours.Add(pto.PrevYearHours).Sub(used).Sub(planned)
	return tx.Save(&pto).Error
}

func createPtoPlanHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		pto, ok := findOwnedPto(w, r, db, principal.UserID, "ptoId")
		if !ok {
			return
		}

		plan, ok := decodePtoPlanRequest(w, r)
		if !ok {
			return
		}
		plan.PtoID = pto.ID

		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Create(&plan).Error; err != nil {
				return err
			}
			return recomputePtoHours(tx, pto.ID)
		})
		if err != nil {
			http.Error(w, "failed to create pto plan", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, toPtoPlanResponse(plan))
	}
}

func findOwnedPtoPlan(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (models.PtoPlan, bool) {
	var plan models.PtoPlan
	ptoID, err := uuid.Parse(r.PathValue("ptoId"))
	if err != nil {
		http.Error(w, "invalid pto id", http.StatusBadRequest)
		return plan, false
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, "invalid id", http.StatusBadRequest)
		return plan, false
	}
	if !ptoOwnedByUser(db, ptoID, userID) {
		http.Error(w, "pto not found", http.StatusNotFound)
		return plan, false
	}
	err = db.Where("id = ? AND pto_id = ?", id, ptoID).First(&plan).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		http.Error(w, "pto plan not found", http.StatusNotFound)
		return plan, false
	}
	if err != nil {
		http.Error(w, "failed to look up pto plan", http.StatusInternalServerError)
		return plan, false
	}
	return plan, true
}

func updatePtoPlanHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		plan, ok := findOwnedPtoPlan(w, r, db, principal.UserID)
		if !ok {
			return
		}

		req, ok := decodePtoPlanRequest(w, r)
		if !ok {
			return
		}

		plan.StartDate = req.StartDate
		plan.EndDate = req.EndDate
		plan.Name = req.Name
		plan.Description = req.Description
		plan.Hours = req.Hours
		plan.Status = req.Status
		plan.CustomHours = req.CustomHours

		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Save(&plan).Error; err != nil {
				return err
			}
			return recomputePtoHours(tx, plan.PtoID)
		})
		if err != nil {
			http.Error(w, "failed to update pto plan", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toPtoPlanResponse(plan))
	}
}

func deletePtoPlanHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		ptoID, err := uuid.Parse(r.PathValue("ptoId"))
		if err != nil {
			http.Error(w, "invalid pto id", http.StatusBadRequest)
			return
		}
		id, err := uuid.Parse(r.PathValue("id"))
		if err != nil {
			http.Error(w, "invalid id", http.StatusBadRequest)
			return
		}
		if !ptoOwnedByUser(db, ptoID, principal.UserID) {
			http.Error(w, "pto not found", http.StatusNotFound)
			return
		}

		err = db.Transaction(func(tx *gorm.DB) error {
			result := tx.Where("id = ? AND pto_id = ?", id, ptoID).Delete(&models.PtoPlan{})
			if result.Error != nil {
				return result.Error
			}
			if result.RowsAffected == 0 {
				return gorm.ErrRecordNotFound
			}
			return recomputePtoHours(tx, ptoID)
		})
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "pto plan not found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to delete pto plan", http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}

// --- HolidayHour: a holiday's hours for a Pto year (nested, informational
// only — does not affect HoursRemaining, matching today's behavior). No
// standalone GET; reads happen via GET /ptos/{id}'s embedded holidayHours.

type holidayHourRequest struct {
	Date  time.Time       `json:"date"`
	Name  string          `json:"name"`
	Hours decimal.Decimal `json:"hours"`
}

type holidayHourResponse struct {
	ID        uuid.UUID       `json:"id"`
	PtoID     uuid.UUID       `json:"ptoId"`
	Date      time.Time       `json:"date"`
	Name      string          `json:"name"`
	Hours     decimal.Decimal `json:"hours"`
	CreatedAt time.Time       `json:"createdAt"`
	UpdatedAt time.Time       `json:"updatedAt"`
}

func toHolidayHourResponse(h models.HolidayHour) holidayHourResponse {
	return holidayHourResponse{
		ID:        h.ID,
		PtoID:     h.PtoID,
		Date:      h.Date,
		Name:      h.Name,
		Hours:     h.Hours,
		CreatedAt: h.CreatedAt,
		UpdatedAt: h.UpdatedAt,
	}
}

func decodeHolidayHourRequest(w http.ResponseWriter, r *http.Request) (models.HolidayHour, bool) {
	var req holidayHourRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return models.HolidayHour{}, false
	}
	if req.Name == "" {
		http.Error(w, "name is required", http.StatusBadRequest)
		return models.HolidayHour{}, false
	}
	if req.Date.IsZero() {
		http.Error(w, "date is required", http.StatusBadRequest)
		return models.HolidayHour{}, false
	}
	return models.HolidayHour{Date: req.Date, Name: req.Name, Hours: req.Hours}, true
}

func createHolidayHourHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		pto, ok := findOwnedPto(w, r, db, principal.UserID, "ptoId")
		if !ok {
			return
		}

		holiday, ok := decodeHolidayHourRequest(w, r)
		if !ok {
			return
		}
		holiday.PtoID = pto.ID

		if err := db.Create(&holiday).Error; err != nil {
			http.Error(w, "failed to create holiday hour", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, toHolidayHourResponse(holiday))
	}
}

func findOwnedHolidayHour(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (models.HolidayHour, bool) {
	var holiday models.HolidayHour
	ptoID, err := uuid.Parse(r.PathValue("ptoId"))
	if err != nil {
		http.Error(w, "invalid pto id", http.StatusBadRequest)
		return holiday, false
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, "invalid id", http.StatusBadRequest)
		return holiday, false
	}
	if !ptoOwnedByUser(db, ptoID, userID) {
		http.Error(w, "pto not found", http.StatusNotFound)
		return holiday, false
	}
	err = db.Where("id = ? AND pto_id = ?", id, ptoID).First(&holiday).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		http.Error(w, "holiday hour not found", http.StatusNotFound)
		return holiday, false
	}
	if err != nil {
		http.Error(w, "failed to look up holiday hour", http.StatusInternalServerError)
		return holiday, false
	}
	return holiday, true
}

func updateHolidayHourHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		holiday, ok := findOwnedHolidayHour(w, r, db, principal.UserID)
		if !ok {
			return
		}

		req, ok := decodeHolidayHourRequest(w, r)
		if !ok {
			return
		}

		holiday.Date = req.Date
		holiday.Name = req.Name
		holiday.Hours = req.Hours

		if err := db.Save(&holiday).Error; err != nil {
			http.Error(w, "failed to update holiday hour", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toHolidayHourResponse(holiday))
	}
}

func deleteHolidayHourHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		ptoID, err := uuid.Parse(r.PathValue("ptoId"))
		if err != nil {
			http.Error(w, "invalid pto id", http.StatusBadRequest)
			return
		}
		id, err := uuid.Parse(r.PathValue("id"))
		if err != nil {
			http.Error(w, "invalid id", http.StatusBadRequest)
			return
		}
		if !ptoOwnedByUser(db, ptoID, principal.UserID) {
			http.Error(w, "pto not found", http.StatusNotFound)
			return
		}

		result := db.Where("id = ? AND pto_id = ?", id, ptoID).Delete(&models.HolidayHour{})
		if result.Error != nil {
			http.Error(w, "failed to delete holiday hour", http.StatusInternalServerError)
			return
		}
		if result.RowsAffected == 0 {
			http.Error(w, "holiday hour not found", http.StatusNotFound)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}
