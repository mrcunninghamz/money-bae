package httpapi

import (
	"errors"
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

// defaultLedgerHistoryLimit matches the home dashboard's "net by cycle"
// chart, which shows 8 bars.
const defaultLedgerHistoryLimit = 8

// checkInThreshold is the free-cash line between a "good" and a "tight"
// bae check-in.
var checkInThreshold = decimal.NewFromInt(100)

type checkInResponse struct {
	Status string `json:"status"`
}

type currentLedgerResponse struct {
	ID             uuid.UUID       `json:"id"`
	Date           time.Time       `json:"date"`
	AvailableFunds decimal.Decimal `json:"availableFunds"`
	Paid           decimal.Decimal `json:"paid"`
	Planned        decimal.Decimal `json:"planned"`
	Net            decimal.Decimal `json:"net"`
	UnpaidCount    int             `json:"unpaidCount"`
	CheckIn        checkInResponse `json:"checkIn"`
}

// checkInStatus classifies free cash (net) into the bae check-in's three
// moods: "good" at $100 or more, "tight" from $0 up to $100, "negative"
// below $0.
func checkInStatus(net decimal.Decimal) string {
	if net.LessThan(decimal.Zero) {
		return "negative"
	}
	if net.LessThan(checkInThreshold) {
		return "tight"
	}
	return "good"
}

// cycleTotals is a ledger's cash position, derived from its stored
// BankBalance/Income fields plus its associated LedgerBills — mirrors the
// frontend's computeCycleTotals (clients/web/src/data/selectors.ts) but
// backed by real rows instead of mock data.
type cycleTotals struct {
	availableFunds decimal.Decimal
	paid           decimal.Decimal
	planned        decimal.Decimal
	net            decimal.Decimal
	unpaidCount    int
}

func computeCycleTotals(ledger models.Ledger) cycleTotals {
	availableFunds := ledger.BankBalance.Add(ledger.Income)
	var paid, planned decimal.Decimal
	unpaidCount := 0
	for _, lb := range ledger.LedgerBills {
		if lb.IsPayed {
			paid = paid.Add(lb.Amount)
		} else {
			planned = planned.Add(lb.Amount)
			unpaidCount++
		}
	}
	return cycleTotals{
		availableFunds: availableFunds,
		paid:           paid,
		planned:        planned,
		net:            availableFunds.Sub(paid).Sub(planned),
		unpaidCount:    unpaidCount,
	}
}

func toCurrentLedgerResponse(ledger models.Ledger) currentLedgerResponse {
	totals := computeCycleTotals(ledger)
	return currentLedgerResponse{
		ID:             ledger.ID,
		Date:           ledger.Date,
		AvailableFunds: totals.availableFunds,
		Paid:           totals.paid,
		Planned:        totals.planned,
		Net:            totals.net,
		UnpaidCount:    totals.unpaidCount,
		CheckIn:        checkInResponse{Status: checkInStatus(totals.net)},
	}
}

func currentLedgerHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		var ledger models.Ledger
		err := db.Where("user_id = ?", principal.UserID).
			Preload("LedgerBills").
			Order("date DESC").
			First(&ledger).Error
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "no cycles found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to look up current cycle", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toCurrentLedgerResponse(ledger))
	}
}

type ledgerHistoryEntry struct {
	ID         uuid.UUID `json:"id"`
	Date       time.Time `json:"date"`
	Name       *string   `json:"name"`
	NetPercent float64   `json:"netPercent"`
}

// netPercent is net as a share of that cycle's available funds — how much
// of the total cash on hand was left over — rather than a raw dollar
// amount, so cycles stay comparable regardless of bank-balance rollover
// size. Reports 0 rather than dividing by zero when there were no
// available funds at all.
func netPercent(ledger models.Ledger) float64 {
	totals := computeCycleTotals(ledger)
	if totals.availableFunds.IsZero() {
		return 0
	}
	pct, _ := totals.net.Div(totals.availableFunds).Mul(decimal.NewFromInt(100)).Float64()
	return pct
}

func toLedgerHistoryEntry(ledger models.Ledger) ledgerHistoryEntry {
	return ledgerHistoryEntry{
		ID:         ledger.ID,
		Date:       ledger.Date,
		Name:       ledger.Name,
		NetPercent: netPercent(ledger),
	}
}

func ledgerHistoryHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		limit := defaultLedgerHistoryLimit
		if limitStr := r.URL.Query().Get("limit"); limitStr != "" {
			parsed, err := strconv.Atoi(limitStr)
			if err != nil || parsed < 1 {
				http.Error(w, "invalid limit", http.StatusBadRequest)
				return
			}
			limit = parsed
		}

		var ledgers []models.Ledger
		err := db.Where("user_id = ?", principal.UserID).
			Preload("LedgerBills").
			Order("date DESC").
			Limit(limit).
			Find(&ledgers).Error
		if err != nil {
			http.Error(w, "failed to list cycle history", http.StatusInternalServerError)
			return
		}

		entries := make([]ledgerHistoryEntry, len(ledgers))
		for i, ledger := range ledgers {
			// ledgers is newest-first (for LIMIT to keep the most recent
			// N); the chart reads oldest-first, so reverse on the way out.
			entries[len(ledgers)-1-i] = toLedgerHistoryEntry(ledger)
		}

		writeJSON(w, http.StatusOK, entries)
	}
}
