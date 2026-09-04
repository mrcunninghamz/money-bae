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

func newLedgerTestRouter(t *testing.T) (http.Handler, *gorm.DB) {
	db := testdb.New(t, &models.User{}, &models.Ledger{}, &models.Bill{}, &models.LedgerBill{}, &models.Income{})
	router := NewRouter(db, auth.MockVerifier{DB: db})
	return router, db
}

func seedLedgerBill(t *testing.T, db *gorm.DB, userID, ledgerID uuid.UUID, amount decimal.Decimal, isPayed bool) models.LedgerBill {
	t.Helper()
	bill := models.Bill{UserID: userID, Name: "test bill", Amount: amount, IsAutoPay: false}
	if err := db.Create(&bill).Error; err != nil {
		t.Fatalf("failed to seed bill: %v", err)
	}
	ledgerBill := models.LedgerBill{LedgerID: ledgerID, BillID: bill.ID, Amount: amount, IsPayed: isPayed}
	if err := db.Create(&ledgerBill).Error; err != nil {
		t.Fatalf("failed to seed ledger bill: %v", err)
	}
	return ledgerBill
}

func decodeCurrentLedgerResponse(t *testing.T, rec *httptest.ResponseRecorder) currentLedgerResponse {
	t.Helper()
	var got currentLedgerResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response %q: %v", rec.Body.String(), err)
	}
	return got
}

func TestCurrentLedger_ComputesTotals(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(1000), Income: decimal.NewFromInt(500), Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	seedLedgerBill(t, db, user.ID, ledger.ID, decimal.NewFromInt(200), true)
	seedLedgerBill(t, db, user.ID, ledger.ID, decimal.NewFromInt(300), false)

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	got := decodeCurrentLedgerResponse(t, rec)
	if !got.AvailableFunds.Equal(decimal.NewFromInt(1500)) {
		t.Fatalf("expected availableFunds 1500, got %s", got.AvailableFunds)
	}
	if !got.Paid.Equal(decimal.NewFromInt(200)) {
		t.Fatalf("expected paid 200, got %s", got.Paid)
	}
	if !got.Planned.Equal(decimal.NewFromInt(300)) {
		t.Fatalf("expected planned 300, got %s", got.Planned)
	}
	if !got.Net.Equal(decimal.NewFromInt(1000)) {
		t.Fatalf("expected net 1000, got %s", got.Net)
	}
	if got.UnpaidCount != 1 {
		t.Fatalf("expected unpaidCount 1, got %d", got.UnpaidCount)
	}
	if got.CheckIn.Status != "good" {
		t.Fatalf("expected status good, got %s", got.CheckIn.Status)
	}
}

func TestCurrentLedger_NoLedgers_Returns404(t *testing.T) {
	router, _ := newLedgerTestRouter(t)

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCurrentLedger_OnlyConsidersOwnLedgers(t *testing.T) {
	router, db := newLedgerTestRouter(t)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	otherLedger := models.Ledger{
		UserID: otherUser.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(1000), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&otherLedger).Error; err != nil {
		t.Fatalf("failed to seed other ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCurrentLedger_PicksMostRecentByDate(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	older := models.Ledger{
		UserID: user.ID, Date: time.Now().AddDate(0, 0, -14),
		BankBalance: decimal.NewFromInt(1), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&older).Error; err != nil {
		t.Fatalf("failed to seed older ledger: %v", err)
	}
	newer := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(2), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&newer).Error; err != nil {
		t.Fatalf("failed to seed newer ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	got := decodeCurrentLedgerResponse(t, rec)
	if got.ID != newer.ID {
		t.Fatalf("expected the newer ledger %s, got %s", newer.ID, got.ID)
	}
}

func TestCurrentLedger_CheckInStatus_TightAtZero(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.Zero, Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	got := decodeCurrentLedgerResponse(t, rec)
	if got.CheckIn.Status != "tight" {
		t.Fatalf("expected status tight at net=0, got %s", got.CheckIn.Status)
	}
}

func TestCurrentLedger_CheckInStatus_GoodAtThreshold(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(100), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	got := decodeCurrentLedgerResponse(t, rec)
	if got.CheckIn.Status != "good" {
		t.Fatalf("expected status good at net=100, got %s", got.CheckIn.Status)
	}
}

func TestCurrentLedger_CheckInStatus_Negative(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	ledger := models.Ledger{
		UserID: user.ID, Date: time.Now(),
		BankBalance: decimal.NewFromInt(-1), Income: decimal.Zero, Expenses: decimal.Zero,
	}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/current", nil)
	got := decodeCurrentLedgerResponse(t, rec)
	if got.CheckIn.Status != "negative" {
		t.Fatalf("expected status negative at net=-1, got %s", got.CheckIn.Status)
	}
}

func seedLedger(t *testing.T, db *gorm.DB, userID uuid.UUID, date time.Time, bank, income decimal.Decimal) models.Ledger {
	t.Helper()
	ledger := models.Ledger{UserID: userID, Date: date, BankBalance: bank, Income: income, Expenses: decimal.Zero}
	if err := db.Create(&ledger).Error; err != nil {
		t.Fatalf("failed to seed ledger: %v", err)
	}
	return ledger
}

func TestLedgerHistory_OrdersOldestToNewestWithNetPercent(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	older := seedLedger(t, db, user.ID, time.Now().AddDate(0, 0, -14), decimal.NewFromInt(100), decimal.Zero)
	newer := seedLedger(t, db, user.ID, time.Now(), decimal.NewFromInt(200), decimal.Zero)
	// older: net=100, availableFunds=100 -> 100%. newer: net=200, availableFunds=200 -> 100%.
	// Give the newer ledger an unpaid bill so its net% differs from the older one.
	seedLedgerBill(t, db, user.ID, newer.ID, decimal.NewFromInt(50), false)

	rec := doJSON(t, router, http.MethodGet, "/ledgers/history", nil)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got []ledgerHistoryEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(got))
	}
	if got[0].ID != older.ID || got[1].ID != newer.ID {
		t.Fatalf("expected oldest-first ordering %s,%s, got %s,%s", older.ID, newer.ID, got[0].ID, got[1].ID)
	}
	if got[0].NetPercent != 100 {
		t.Fatalf("expected older netPercent 100, got %v", got[0].NetPercent)
	}
	if got[1].NetPercent != 75 {
		t.Fatalf("expected newer netPercent 75, got %v", got[1].NetPercent)
	}
}

func TestLedgerHistory_ZeroAvailableFunds_NetPercentZero(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)
	seedLedger(t, db, user.ID, time.Now(), decimal.Zero, decimal.Zero)

	rec := doJSON(t, router, http.MethodGet, "/ledgers/history", nil)
	var got []ledgerHistoryEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 1 || got[0].NetPercent != 0 {
		t.Fatalf("expected a single zero-percent entry, got %+v", got)
	}
}

func TestLedgerHistory_RespectsLimitParam(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)

	for i := 0; i < 5; i++ {
		seedLedger(t, db, user.ID, time.Now().AddDate(0, 0, -i), decimal.NewFromInt(1), decimal.Zero)
	}

	rec := doJSON(t, router, http.MethodGet, "/ledgers/history?limit=2", nil)
	var got []ledgerHistoryEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("expected 2 entries with limit=2, got %d", len(got))
	}
}

func TestLedgerHistory_OnlyReturnsOwnLedgers(t *testing.T) {
	router, db := newLedgerTestRouter(t)
	user := seedCurrentUser(t, db)
	seedLedger(t, db, user.ID, time.Now(), decimal.NewFromInt(1), decimal.Zero)

	otherUser := models.User{Sub: "auth0|other", Email: "other@example.com"}
	if err := db.Create(&otherUser).Error; err != nil {
		t.Fatalf("failed to seed other user: %v", err)
	}
	seedLedger(t, db, otherUser.ID, time.Now(), decimal.NewFromInt(999), decimal.Zero)

	rec := doJSON(t, router, http.MethodGet, "/ledgers/history", nil)
	var got []ledgerHistoryEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(got) != 1 {
		t.Fatalf("expected only the current user's ledger, got %+v", got)
	}
}
