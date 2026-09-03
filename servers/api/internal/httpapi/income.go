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

type incomeRequest struct {
	Date     time.Time       `json:"date"`
	Amount   decimal.Decimal `json:"amount"`
	LedgerID *uuid.UUID      `json:"ledgerId"`
	Notes    *string         `json:"notes"`
}

type incomeResponse struct {
	ID        uuid.UUID       `json:"id"`
	Date      time.Time       `json:"date"`
	Amount    decimal.Decimal `json:"amount"`
	LedgerID  *uuid.UUID      `json:"ledgerId"`
	Notes     *string         `json:"notes"`
	CreatedAt time.Time       `json:"createdAt"`
	UpdatedAt time.Time       `json:"updatedAt"`
}

func toIncomeResponse(income models.Income) incomeResponse {
	return incomeResponse{
		ID:        income.ID,
		Date:      income.Date,
		Amount:    income.Amount,
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

func decodeIncomeRequest(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (incomeRequest, bool) {
	var req incomeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return req, false
	}
	if req.Date.IsZero() {
		http.Error(w, "date is required", http.StatusBadRequest)
		return req, false
	}
	if req.LedgerID != nil && !ledgerOwnedByUser(db, *req.LedgerID, userID) {
		http.Error(w, "ledger not found", http.StatusBadRequest)
		return req, false
	}
	return req, true
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
		if err := db.Create(&income).Error; err != nil {
			http.Error(w, "failed to create income", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, toIncomeResponse(income))
	}
}

func listIncomesHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		var incomes []models.Income
		if err := db.Where("user_id = ?", principal.UserID).Find(&incomes).Error; err != nil {
			http.Error(w, "failed to list incomes", http.StatusInternalServerError)
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
		http.Error(w, "failed to look up income", http.StatusInternalServerError)
		return income, false
	}
	return income, true
}

func getIncomeHandler(db *gorm.DB) http.HandlerFunc {
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

		writeJSON(w, http.StatusOK, toIncomeResponse(income))
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

		income.Date = req.Date
		income.Amount = req.Amount
		income.LedgerID = req.LedgerID
		income.Notes = req.Notes

		if err := db.Save(&income).Error; err != nil {
			http.Error(w, "failed to update income", http.StatusInternalServerError)
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

		id, err := uuid.Parse(r.PathValue("id"))
		if err != nil {
			http.Error(w, "invalid id", http.StatusBadRequest)
			return
		}

		result := db.Where("id = ? AND user_id = ?", id, principal.UserID).Delete(&models.Income{})
		if result.Error != nil {
			http.Error(w, "failed to delete income", http.StatusInternalServerError)
			return
		}
		if result.RowsAffected == 0 {
			http.Error(w, "income not found", http.StatusNotFound)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}
