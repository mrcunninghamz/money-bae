package httpapi

import (
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

func newLedgerTestRouter(t *testing.T) (http.Handler, *gorm.DB) {
	db := testdb.New(t, &models.User{}, &models.Ledger{}, &models.Income{}, &models.Bill{}, &models.LedgerBill{})
	router := NewRouter(db, auth.MockVerifier{DB: db})
	return router, db
}

func decodeLedgerResponse(t *testing.T, rec *httptest.ResponseRecorder) ledgerResponse {
	t.Helper()
	var got ledgerResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	return got
}

func TestCreateLedger_MinimalFields_DefaultsAndComputesNet(t *testing.T) {
	router, _ := newLedgerTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/ledgers", ledgerRequest{
		Date: time.Date(2025, 12, 12, 0, 0, 0, 0, time.UTC),
		Name: strPtr("December P1"),
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeLedgerResponse(t, rec)
	if got.BankBalance.Amount() != 0 || got.Income.Amount() != 0 || got.Expenses.Amount() != 0 {
		t.Fatalf("expected zero-defaulted money fields, got %+v", got)
	}
	if got.Net.Amount() != 0 {
		t.Fatalf("expected net 0, got %+v", got.Net)
	}
}

func TestCreateLedger_ComputesNetFromBankBalanceIncomeExpenses(t *testing.T) {
	router, _ := newLedgerTestRouter(t)

	bankBalance := money.New(500000, "USD")
	income := money.New(300000, "USD")
	expenses := money.New(200000, "USD")
	rec := doJSON(t, router, http.MethodPost, "/ledgers", ledgerRequest{
		Date:        time.Now(),
		BankBalance: bankBalance,
		Income:      income,
		Expenses:    expenses,
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeLedgerResponse(t, rec)
	if got.Net.Amount() != 600000 {
		t.Fatalf("expected net 6000 USD (5000+3000-2000), got %+v", got.Net)
	}
}

func TestCreateLedger_MissingDate_Returns400(t *testing.T) {
	router, _ := newLedgerTestRouter(t)

	rec := doJSON(t, router, http.MethodPost, "/ledgers", ledgerRequest{})
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestListLedgers_OrderParam_DefaultsNewestFirst(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	older := models.Ledger{
		UserID: user.ID, Date: time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&older).Error; err != nil {
		t.Fatalf("failed to seed older ledger: %v", err)
	}
	newer := models.Ledger{
		UserID: user.ID, Date: time.Date(2025, 12, 1, 0, 0, 0, 0, time.UTC),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&newer).Error; err != nil {
		t.Fatalf("failed to seed newer ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers", nil)
	var got []ledgerResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != newer.ID || got[1].ID != older.ID {
		t.Fatalf("expected [newer, older] by default, got %+v", got)
	}

	rec = doJSON(t, router, http.MethodGet, "/ledgers?order=asc", nil)
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 || got[0].ID != older.ID || got[1].ID != newer.ID {
		t.Fatalf("expected [older, newer] with order=asc, got %+v", got)
	}
}

func TestGetLedger_IncludesIncomesAndLedgerBillsWithBill(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(1000), LedgerID: &ledger.ID}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}
	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
	lb := models.LedgerBill{LedgerID: ledger.ID, BillID: bill.ID, Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&lb).Error; err != nil {
		t.Fatalf("failed to seed ledger bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/"+ledger.ID.String(), nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got ledgerDetailResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if len(got.Incomes) != 1 || got.Incomes[0].ID != income.ID {
		t.Fatalf("expected 1 nested income, got %+v", got.Incomes)
	}
	if len(got.LedgerBills) != 1 || got.LedgerBills[0].Bill.ID != bill.ID {
		t.Fatalf("expected 1 nested ledger bill with bill %s, got %+v", bill.ID, got.LedgerBills)
	}
}

func TestGetLedger_BelongsToAnotherUser_Returns404(t *testing.T) {
	router, db := newLedgerTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	ledger := models.Ledger{
		UserID: otherUser.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/"+ledger.ID.String(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestUpdateLedger_Succeeds(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(), Name: strPtr("Old"),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodPut, "/ledgers/"+ledger.ID.String(), ledgerRequest{
		Date: time.Now(),
		Name: strPtr("New"),
	})

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeLedgerResponse(t, rec)
	if got.Name == nil || *got.Name != "New" {
		t.Fatalf("expected updated name, got %+v", got.Name)
	}
}

func TestDeleteLedger_NullsIncomeLedgerIdAndCascadesLedgerBills(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	income := models.Income{UserID: user.ID, Date: time.Now(), Amount: decimal.NewFromInt(1000), LedgerID: &ledger.ID}
	if err := db.Create(&income).Error; err != nil {
		t.Fatalf("failed to seed income: %v", err)
	}
	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
	lb := models.LedgerBill{LedgerID: ledger.ID, BillID: bill.ID, Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&lb).Error; err != nil {
		t.Fatalf("failed to seed ledger bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ledgers/"+ledger.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var reloadedIncome models.Income
	if err := db.First(&reloadedIncome, "id = ?", income.ID).Error; err != nil {
		t.Fatalf("expected income to survive ledger delete: %v", err)
	}
	if reloadedIncome.LedgerID != nil {
		t.Fatalf("expected income's ledgerId to be nulled, got %+v", reloadedIncome.LedgerID)
	}

	var ledgerBillCount int64
	db.Model(&models.LedgerBill{}).Where("ledger_id = ?", ledger.ID).Count(&ledgerBillCount)
	if ledgerBillCount != 0 {
		t.Fatalf("expected ledger bill to be cascade-deleted, found %d rows", ledgerBillCount)
	}
}

// --- LedgerBill (nested)

func TestCreateLedgerBill_Succeeds(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ledgers/"+ledger.ID.String()+"/bills", ledgerBillRequest{
		BillID: bill.ID,
		Amount: *money.New(150000, "USD"),
	})

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var got ledgerBillResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if got.LedgerID != ledger.ID || got.BillID != bill.ID {
		t.Fatalf("expected ledgerId %s and billId %s, got %+v", ledger.ID, bill.ID, got)
	}
}

func TestCreateLedgerBill_BillNotOwnedByUser_Returns400(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	otherBill := models.Bill{UserID: otherUser.ID, Name: "Other", Amount: decimal.NewFromInt(10)}
	if err := db.Create(&otherBill).Error; err != nil {
		t.Fatalf("failed to seed other bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodPost, "/ledgers/"+ledger.ID.String()+"/bills", ledgerBillRequest{
		BillID: otherBill.ID,
		Amount: *money.New(10000, "USD"),
	})

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCreateLedgerBill_LedgerNotOwnedByUser_Returns404(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
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

	rec := doJSON(t, router, http.MethodPost, "/ledgers/"+otherLedger.ID.String()+"/bills", ledgerBillRequest{
		BillID: bill.ID,
		Amount: *money.New(150000, "USD"),
	})

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestUpdateLedgerBill_TogglesPaid(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
	lb := models.LedgerBill{LedgerID: ledger.ID, BillID: bill.ID, Amount: decimal.NewFromInt(150000), IsPayed: false}
	if err := db.Create(&lb).Error; err != nil {
		t.Fatalf("failed to seed ledger bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodPut, "/ledgers/"+ledger.ID.String()+"/bills/"+lb.ID.String(), ledgerBillRequest{
		BillID:  bill.ID,
		Amount:  *money.New(150000, "USD"),
		IsPayed: true,
	})

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var got ledgerBillResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	if !got.IsPayed {
		t.Fatalf("expected isPayed true, got %+v", got)
	}
}

func TestDeleteLedgerBill_Succeeds(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	bill := models.Bill{UserID: user.ID, Name: "Mortgage", Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
	lb := models.LedgerBill{LedgerID: ledger.ID, BillID: bill.ID, Amount: decimal.NewFromInt(150000)}
	if err := db.Create(&lb).Error; err != nil {
		t.Fatalf("failed to seed ledger bill: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ledgers/"+ledger.ID.String()+"/bills/"+lb.ID.String(), nil)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	var count int64
	db.Model(&models.LedgerBill{}).Where("id = ?", lb.ID).Count(&count)
	if count != 0 {
		t.Fatalf("expected ledger bill to be deleted, found %d rows", count)
	}
}

func TestDeleteLedgerBill_NotFound_Returns404(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodDelete, "/ledgers/"+ledger.ID.String()+"/bills/"+uuid.NewString(), nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}
