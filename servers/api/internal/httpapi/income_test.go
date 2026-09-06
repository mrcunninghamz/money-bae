package httpapi

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	money "github.com/Rhymond/go-money"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func newIncomeTestRouter(t *testing.T) (http.Handler, *gorm.DB) {
	db := testdb.New(t, &models.User{}, &models.Ledger{}, &models.Income{}, &models.LedgerBill{}, &models.Bill{})
	router := NewRouter(db, auth.MockVerifier{DB: db})
	return router, db
}

// seedCurrentUser provisions the seed user row up front (MockVerifier would
// otherwise lazily create it on first request) so tests can attach owned
// fixtures like a ledger before making any HTTP call.
func seedCurrentUser(t *testing.T, db *gorm.DB) models.User {
	t.Helper()
	user := models.User{Sub: auth.SeedUserSub, Email: auth.SeedUserEmail}
	if err := db.Create(&user).Error; err != nil {
		t.Fatalf("failed to seed current user: %v", err)
	}
	return user
}

func doJSON(t *testing.T, router http.Handler, method, path string, body any) *httptest.ResponseRecorder {
	t.Helper()
	reqBody := &bytes.Buffer{}
	if body != nil {
		if err := json.NewEncoder(reqBody).Encode(body); err != nil {
			t.Fatalf("failed to encode request body: %v", err)
		}
	}
	req := httptest.NewRequest(method, path, reqBody)
	req.Header.Set("Authorization", "Bearer anything")
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	return rec
}

func decodeIncomeResponse(t *testing.T, rec *httptest.ResponseRecorder) incomeResponse {
	t.Helper()
	var got incomeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	return got
}

func TestCreateIncome_Succeeds(t *testing.T) {
	router, _ := newIncomeTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:   time.Date(2026, 1, 15, 0, 0, 0, 0, time.UTC),
		Amount: *money.New(150000, "USD"),
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeIncomeResponse(t, rec)
	if got.ID == uuid.Nil {
		t.Fatal("expected a generated id")
	}
	if got.Amount.Amount() != 150000 {
		t.Fatalf("expected amount 1500 USD, got %+v", got.Amount)
	}
}

func TestCreateIncome_MissingDate_Returns400(t *testing.T) {
	router, _ := newIncomeTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Amount: *money.New(150000, "USD"),
	})

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestCreateIncome_MissingAmount_Returns400(t *testing.T) {
	router, _ := newIncomeTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date: time.Now(),
	})

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestCreateIncome_LedgerNotOwnedByUser_Returns400(t *testing.T) {
	router, db := newIncomeTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	otherLedger := models.Ledger{
		UserID: otherUser.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&otherLedger).Error; err != nil {
		t.Fatalf("failed to seed other ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(10000, "USD"),
		LedgerID: &otherLedger.ID,
	})

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCreateIncome_WithOwnLedger_Succeeds(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(10000, "USD"),
		LedgerID: &ledger.ID,
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeIncomeResponse(t, rec)
	if got.LedgerID == nil || *got.LedgerID != ledger.ID {
		t.Fatalf("expected ledgerId %s, got %+v", ledger.ID, got.LedgerID)
	}
}

func TestCreateIncome_WithLedger_RecalculatesLedgerTotals(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(1000), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(20000, "USD"), // $200
		LedgerID: &ledger.ID,
	})
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = doJSON(t, router, http.MethodGet, "/ledgers/"+ledger.ID.String(), nil)
	got := decodeLedgerResponse(t, rec)
	if got.Income.Amount() != 20000 {
		t.Fatalf("expected ledger income 200 USD, got %+v", got.Income)
	}
	if got.Net.Amount() != 120000 {
		t.Fatalf("expected net 1200 USD (1000 bank + 200 income), got %+v", got.Net)
	}
}

func TestUpdateIncome_MovedToDifferentLedger_RecalculatesBothLedgers(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	ledgerA := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(1000), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledgerA).Error; err != nil {
		t.Fatalf("failed to seed ledger A: %v", err)
	}
	ledgerB := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(500), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledgerB).Error; err != nil {
		t.Fatalf("failed to seed ledger B: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(20000, "USD"), // $200
		LedgerID: &ledgerA.ID,
	})
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	income := decodeIncomeResponse(t, rec)

	rec = doJSON(t, router, http.MethodPut, "/incomes/"+income.ID.String(), incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(20000, "USD"),
		LedgerID: &ledgerB.ID,
	})
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = doJSON(t, router, http.MethodGet, "/ledgers/"+ledgerA.ID.String(), nil)
	gotA := decodeLedgerResponse(t, rec)
	if gotA.Income.Amount() != 0 {
		t.Fatalf("expected ledger A income back to 0, got %+v", gotA.Income)
	}
	if gotA.Net.Amount() != 100000 {
		t.Fatalf("expected ledger A net back to 1000 USD, got %+v", gotA.Net)
	}

	rec = doJSON(t, router, http.MethodGet, "/ledgers/"+ledgerB.ID.String(), nil)
	gotB := decodeLedgerResponse(t, rec)
	if gotB.Income.Amount() != 20000 {
		t.Fatalf("expected ledger B income 200 USD, got %+v", gotB.Income)
	}
	if gotB.Net.Amount() != 70000 {
		t.Fatalf("expected ledger B net 700 USD (500 bank + 200 income), got %+v", gotB.Net)
	}
}

func TestDeleteIncome_RecalculatesLedgerTotals(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(1000), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/incomes", incomeRequest{
		Date:     time.Now(),
		Amount:   *money.New(20000, "USD"),
		LedgerID: &ledger.ID,
	})
	income := decodeIncomeResponse(t, rec)

	rec = doJSON(t, router, http.MethodDelete, "/incomes/"+income.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = doJSON(t, router, http.MethodGet, "/ledgers/"+ledger.ID.String(), nil)
	got := decodeLedgerResponse(t, rec)
	if got.Income.Amount() != 0 {
		t.Fatalf("expected ledger income back to 0, got %+v", got.Income)
	}
	if got.Net.Amount() != 100000 {
		t.Fatalf("expected net back to 1000 USD, got %+v", got.Net)
	}
}

func TestListIncomes_OnlyReturnsCurrentUsersIncomes(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	mine := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&mine).Error; err != nil {
		t.Fatalf("failed to seed own income: %v", err)
	}

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	other := models.Income{UserID: otherUser.ID, Date: time.Now(), Amount: decimal.NewFromInt(20)}
	if err := db.Create(&other).Error; err != nil {
		t.Fatalf("failed to seed other income: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/incomes", nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got []incomeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 1 || got[0].ID != mine.ID {
		t.Fatalf("expected exactly the current user's income, got %+v", got)
	}
}

func TestListIncomes_OrderParam_ReversesOrder(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	older := models.Income{UserID: user.ID, Date: time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&older).Error; err != nil {
		t.Fatalf("failed to seed older income: %v", err)
	}
	newer := models.Income{UserID: user.ID, Date: time.Date(2025, 12, 1, 0, 0, 0, 0, time.UTC), Amount: decimal.NewFromInt(20)}
	if err := db.Create(&newer).Error; err != nil {
		t.Fatalf("failed to seed newer income: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/incomes", nil)
	var got []incomeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != newer.ID || got[1].ID != older.ID {
		t.Fatalf("expected [newer, older] by default, got %+v", got)
	}

	rec = doJSON(t, router, http.MethodGet, "/incomes?order=asc", nil)
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != older.ID || got[1].ID != newer.ID {
		t.Fatalf("expected [older, newer] with order=asc, got %+v", got)
	}
}

func TestGetIncome_IncludesNestedLedger(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(), Name: strPtr("December P1"),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(10), LedgerID: &ledger.ID}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/incomes/"+income.ID.String(), nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got incomeDetailResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if got.Ledger == nil || got.Ledger.ID != ledger.ID {
		t.Fatalf("expected nested ledger %s, got %+v", ledger.ID, got.Ledger)
	}
}

func TestGetIncome_Succeeds(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/incomes/"+income.ID.String(), nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeIncomeResponse(t, rec)
	if got.ID != income.ID {
		t.Fatalf("expected income %s, got %s", income.ID, got.ID)
	}
}

func TestGetIncome_NotFound_Returns404(t *testing.T) {
	router, _ := newIncomeTestRouter(t)

	rec := doJSON(t, router, http.MethodGet, "/incomes/"+uuid.NewString(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestGetIncome_BelongsToAnotherUser_Returns404(t *testing.T) {
	router, db := newIncomeTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	income := models.Income{UserID: otherUser.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/incomes/"+income.ID.String(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestUpdateIncome_Succeeds(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	newDate := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	rec := doJSON(t, router, http.MethodPut, "/incomes/"+income.ID.String(), incomeRequest{
		Date:   newDate,
		Amount: *money.New(99900, "USD"),
	})

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeIncomeResponse(t, rec)
	if got.Amount.Amount() != 99900 {
		t.Fatalf("expected amount 999 USD, got %+v", got.Amount)
	}
	if !got.Date.Equal(newDate) {
		t.Fatalf("expected date %s, got %s", newDate, got.Date)
	}
}

func TestUpdateIncome_BelongsToAnotherUser_Returns404(t *testing.T) {
	router, db := newIncomeTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	income := models.Income{UserID: otherUser.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	rec := doJSON(t, router, http.MethodPut, "/incomes/"+income.ID.String(), incomeRequest{
		Date:   time.Now(),
		Amount: *money.New(99900, "USD"),
	})

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestDeleteIncome_Succeeds(t *testing.T) {
	router, db := newIncomeTestRouter(t)
	user := seedCurrentUser(t, db)

	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(10)}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/incomes/"+income.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var count int64
	db.Model(&models.Income{}).Where("id = ?", income.ID).Count(&count)
	if count != 0 {
		t.Fatalf("expected income to be deleted, found %d rows", count)
	}
}

func TestDeleteIncome_NotFound_Returns404(t *testing.T) {
	router, _ := newIncomeTestRouter(t)

	rec := doJSON(t, router, http.MethodDelete, "/incomes/"+uuid.NewString(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}
