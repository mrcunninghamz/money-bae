package httpapi

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
	"time"

	money "github.com/Rhymond/go-money"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type incomeRequest struct {
	Date     time.Time   `json:"date"`
	Amount   money.Money `json:"amount"`
	LedgerID *uuid.UUID  `json:"ledgerId"`
	Notes    *string     `json:"notes"`
}

type incomeResponse struct {
	ID        uuid.UUID   `json:"id"`
	Date      time.Time   `json:"date"`
	Amount    money.Money `json:"amount"`
	LedgerID  *uuid.UUID  `json:"ledgerId"`
	Notes     *string     `json:"notes"`
	CreatedAt time.Time   `json:"createdAt"`
	UpdatedAt time.Time   `json:"updatedAt"`
}

// validatedIncome is a decoded, fully-validated incomeRequest, with Amount
// already converted to decimal.Decimal for storage.
type validatedIncome struct {
	Date     time.Time
	Amount   decimal.Decimal
	LedgerID *uuid.UUID
	Notes    *string
}

func toIncomeResponse(income models.Income) incomeResponse {
	return incomeResponse{
		ID:        income.ID,
		Date:      income.Date,
		Amount:    decimalToMoney(income.Amount),
		LedgerID:  income.LedgerID,
		Notes:     income.Notes,
		CreatedAt: income.CreatedAt,
		UpdatedAt: income.UpdatedAt,
	}
}

// ledgerOwnedByUser reports whether a ledger with the given id exists and
// belongs to userID — checked before attaching an income to it, so one
// user can never reference another user's ledger.
func ledgerOwnedByUser(db *gorm.DB, ledgerID, userID uuid.UUID) bool {
	var count int64
	db.Model(&models.Ledger{}).Where("id = ? AND user_id = ?", ledgerID, userID).Count(&count)
	return count > 0
}

func decodeIncomeRequest(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (validatedIncome, bool) {
	var req incomeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return validatedIncome{}, false
	}
	if req.Date.IsZero() {
		http.Error(w, "date is required", http.StatusBadRequest)
		return validatedIncome{}, false
	}
	amount, ok := moneyToDecimal(req.Amount)
	if !ok {
		http.Error(w, "amount is required, in USD", http.StatusBadRequest)
		return validatedIncome{}, false
	}
	if req.LedgerID != nil && !ledgerOwnedByUser(db, *req.LedgerID, userID) {
		http.Error(w, "ledger not found", http.StatusBadRequest)
		return validatedIncome{}, false
	}
	return validatedIncome{Date: req.Date, Amount: amount, LedgerID: req.LedgerID, Notes: req.Notes}, true
}

func writeJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(body)
}

func createIncomeHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		req, ok := decodeIncomeRequest(w, r, db, principal.UserID)
		if !ok {
			return
		}

		income := models.Income{
			UserID:   principal.UserID,
			Date:     req.Date,
			Amount:   req.Amount,
			LedgerID: req.LedgerID,
			Notes:    req.Notes,
		}

		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Create(&income).Error; err != nil {
				return err
			}
			if income.LedgerID != nil {
				return recalculateLedgerTotals(tx, *income.LedgerID)
			}
			return nil
		})
		if err != nil {
			internalError(w, r, "failed to create income", err)
			return
		}

		writeJSON(w, http.StatusCreated, toIncomeResponse(income))
	}
}

// applyIncomeDateFilter narrows a query by the request's date filter:
// ?year=YYYY, or ?from=/?to= (either or both, RFC3339), or neither for all
// dates. year takes precedence if both are given.
func applyIncomeDateFilter(query *gorm.DB, r *http.Request) (*gorm.DB, error) {
	params := r.URL.Query()

	if yearStr := params.Get("year"); yearStr != "" {
		year, err := strconv.Atoi(yearStr)
		if err != nil {
			return nil, fmt.Errorf("invalid year: %q", yearStr)
		}
		from := time.Date(year, time.January, 1, 0, 0, 0, 0, time.UTC)
		to := time.Date(year+1, time.January, 1, 0, 0, 0, 0, time.UTC)
		return query.Where("date >= ? AND date < ?", from, to), nil
	}

	if fromStr := params.Get("from"); fromStr != "" {
		from, err := time.Parse(time.RFC3339, fromStr)
		if err != nil {
			return nil, fmt.Errorf("invalid from: %q", fromStr)
		}
		query = query.Where("date >= ?", from)
	}
	if toStr := params.Get("to"); toStr != "" {
		to, err := time.Parse(time.RFC3339, toStr)
		if err != nil {
			return nil, fmt.Errorf("invalid to: %q", toStr)
		}
		query = query.Where("date <= ?", to)
	}
	return query, nil
}

func listIncomesHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		query, err := applyIncomeDateFilter(db.Where("user_id = ?", principal.UserID), r)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		var incomes []models.Income
		order := "date " + orderDirection(r)
		if err := query.Order(order).Find(&incomes).Error; err != nil {
			internalError(w, r, "failed to list incomes", err)
			return
		}

		responses := make([]incomeResponse, len(incomes))
		for i, income := range incomes {
			responses[i] = toIncomeResponse(income)
		}

		writeJSON(w, http.StatusOK, responses)
	}
}

func findOwnedIncome(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (models.Income, bool) {
	var income models.Income
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, "invalid id", http.StatusBadRequest)
		return income, false
	}

	err = db.Where("id = ? AND user_id = ?", id, userID).First(&income).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		http.Error(w, "income not found", http.StatusNotFound)
		return income, false
	}
	if err != nil {
		internalError(w, r, "failed to look up income", err)
		return income, false
	}
	return income, true
}

// incomeDetailResponse is the "detail" shape (GET /incomes/{id} only): the
// income's own fields plus its navigational property — the ledger it's
// attached to, if any. List/create/update stay lean (ledgerId only).
type incomeDetailResponse struct {
	incomeResponse
	Ledger *ledgerResponse `json:"ledger"`
}

func toIncomeDetailResponse(income models.Income) incomeDetailResponse {
	var ledger *ledgerResponse
	if income.Ledger != nil {
		l := toLedgerResponse(*income.Ledger)
		ledger = &l
	}
	return incomeDetailResponse{
		incomeResponse: toIncomeResponse(income),
		Ledger:         ledger,
	}
}

func getIncomeHandler(db *gorm.DB) http.HandlerFunc {
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

		var income models.Income
		err = db.Preload("Ledger").Where("id = ? AND user_id = ?", id, principal.UserID).First(&income).Error
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "income not found", http.StatusNotFound)
			return
		}
		if err != nil {
			internalError(w, r, "failed to look up income", err)
			return
		}

		writeJSON(w, http.StatusOK, toIncomeDetailResponse(income))
	}
}

func updateIncomeHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		income, ok := findOwnedIncome(w, r, db, principal.UserID)
		if !ok {
			return
		}

		req, ok := decodeIncomeRequest(w, r, db, principal.UserID)
		if !ok {
			return
		}

		oldLedgerID := income.LedgerID
		income.Date = req.Date
		income.Amount = req.Amount
		income.LedgerID = req.LedgerID
		income.Notes = req.Notes

		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Save(&income).Error; err != nil {
				return err
			}
			if income.LedgerID != nil {
				if err := recalculateLedgerTotals(tx, *income.LedgerID); err != nil {
					return err
				}
			}
			// The income moved to a different ledger (or was detached) —
			// the ledger it left behind needs recalculating too, same as
			// tui/'s trigger did when ledger_id changed on an UPDATE.
			if oldLedgerID != nil && (income.LedgerID == nil || *oldLedgerID != *income.LedgerID) {
				return recalculateLedgerTotals(tx, *oldLedgerID)
			}
			return nil
		})
		if err != nil {
			internalError(w, r, "failed to update income", err)
			return
		}

		writeJSON(w, http.StatusOK, toIncomeResponse(income))
	}
}

func deleteIncomeHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		income, ok := findOwnedIncome(w, r, db, principal.UserID)
		if !ok {
			return
		}

		err := db.Transaction(func(tx *gorm.DB) error {
			if err := tx.Delete(&income).Error; err != nil {
				return err
			}
			if income.LedgerID != nil {
				return recalculateLedgerTotals(tx, *income.LedgerID)
			}
			return nil
		})
		if err != nil {
			internalError(w, r, "failed to delete income", err)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}
