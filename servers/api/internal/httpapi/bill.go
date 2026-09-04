package httpapi

import (
	"encoding/json"
	"errors"
	"net/http"
	"time"

	money "github.com/Rhymond/go-money"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type billRequest struct {
	Name      string      `json:"name"`
	Amount    money.Money `json:"amount"`
	DueDay    *int        `json:"dueDay"`
	IsAutoPay bool        `json:"isAutoPay"`
	Notes     *string     `json:"notes"`
}

// validatedBill is a decoded, fully-validated billRequest, with Amount and
// DueDay already converted to storage types.
type validatedBill struct {
	Name      string
	Amount    decimal.Decimal
	DueDay    *time.Time
	IsAutoPay bool
	Notes     *string
}

// dueDayToTime/timeToDueDay bridge Bill.DueDay's storage type (*time.Time)
// to the API's day-of-month integer (1-31) — the model wasn't changed for
// this, since a real calendar date carries no useful information for a
// recurring bill's due day.
func dueDayToTime(day *int) *time.Time {
	if day == nil {
		return nil
	}
	t := time.Date(1, 1, *day, 0, 0, 0, 0, time.UTC)
	return &t
}

func timeToDueDay(t *time.Time) *int {
	if t == nil {
		return nil
	}
	day := t.Day()
	return &day
}

func decodeBillRequest(w http.ResponseWriter, r *http.Request) (validatedBill, bool) {
	var req billRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid request body", http.StatusBadRequest)
		return validatedBill{}, false
	}
	if req.Name == "" {
		http.Error(w, "name is required", http.StatusBadRequest)
		return validatedBill{}, false
	}
	amount, ok := moneyToDecimal(req.Amount)
	if !ok {
		http.Error(w, "amount is required, in USD", http.StatusBadRequest)
		return validatedBill{}, false
	}
	if req.DueDay != nil && (*req.DueDay < 1 || *req.DueDay > 31) {
		http.Error(w, "dueDay must be between 1 and 31", http.StatusBadRequest)
		return validatedBill{}, false
	}
	return validatedBill{
		Name:      req.Name,
		Amount:    amount,
		DueDay:    dueDayToTime(req.DueDay),
		IsAutoPay: req.IsAutoPay,
		Notes:     req.Notes,
	}, true
}

type billResponse struct {
	ID        uuid.UUID   `json:"id"`
	Name      string      `json:"name"`
	Amount    money.Money `json:"amount"`
	DueDay    *int        `json:"dueDay"`
	IsAutoPay bool        `json:"isAutoPay"`
	Notes     *string     `json:"notes"`
	CreatedAt time.Time   `json:"createdAt"`
	UpdatedAt time.Time   `json:"updatedAt"`
}

func toBillResponse(bill models.Bill) billResponse {
	return billResponse{
		ID:        bill.ID,
		Name:      bill.Name,
		Amount:    decimalToMoney(bill.Amount),
		DueDay:    timeToDueDay(bill.DueDay),
		IsAutoPay: bill.IsAutoPay,
		Notes:     bill.Notes,
		CreatedAt: bill.CreatedAt,
		UpdatedAt: bill.UpdatedAt,
	}
}

// billDetailResponse is the "detail" shape (GET /bills/{id} only): the
// bill's own fields plus every cycle it's been linked into, each with its
// own ledger — a bill's usage history. List/create/update stay lean.
type billDetailResponse struct {
	billResponse
	LedgerBills []ledgerBillWithLedger `json:"ledgerBills"`
}

func toBillDetailResponse(bill models.Bill) billDetailResponse {
	ledgerBills := make([]ledgerBillWithLedger, len(bill.LedgerBills))
	for i, lb := range bill.LedgerBills {
		ledgerBills[i] = toLedgerBillWithLedger(lb)
	}
	return billDetailResponse{
		billResponse: toBillResponse(bill),
		LedgerBills:  ledgerBills,
	}
}

// billOwnedByUser reports whether a bill with the given id exists and
// belongs to userID — checked before linking it into a ledger cycle.
func billOwnedByUser(db *gorm.DB, billID, userID uuid.UUID) bool {
	var count int64
	db.Model(&models.Bill{}).Where("id = ? AND user_id = ?", billID, userID).Count(&count)
	return count > 0
}

func findOwnedBill(w http.ResponseWriter, r *http.Request, db *gorm.DB, userID uuid.UUID) (models.Bill, bool) {
	var bill models.Bill
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		http.Error(w, "invalid id", http.StatusBadRequest)
		return bill, false
	}

	err = db.Where("id = ? AND user_id = ?", id, userID).First(&bill).Error
	if errors.Is(err, gorm.ErrRecordNotFound) {
		http.Error(w, "bill not found", http.StatusNotFound)
		return bill, false
	}
	if err != nil {
		http.Error(w, "failed to look up bill", http.StatusInternalServerError)
		return bill, false
	}
	return bill, true
}

func createBillHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		req, ok := decodeBillRequest(w, r)
		if !ok {
			return
		}

		bill := models.Bill{
			UserID:    principal.UserID,
			Name:      req.Name,
			Amount:    req.Amount,
			DueDay:    req.DueDay,
			IsAutoPay: req.IsAutoPay,
			Notes:     req.Notes,
		}

		if err := db.Create(&bill).Error; err != nil {
			http.Error(w, "failed to create bill", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusCreated, toBillResponse(bill))
	}
}

func listBillsHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		var bills []models.Bill
		order := "created_at " + orderDirection(r)
		if err := db.Where("user_id = ?", principal.UserID).Order(order).Find(&bills).Error; err != nil {
			http.Error(w, "failed to list bills", http.StatusInternalServerError)
			return
		}

		responses := make([]billResponse, len(bills))
		for i, bill := range bills {
			responses[i] = toBillResponse(bill)
		}

		writeJSON(w, http.StatusOK, responses)
	}
}

func getBillHandler(db *gorm.DB) http.HandlerFunc {
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

		var bill models.Bill
		err = db.Preload("LedgerBills.Ledger").
			Where("id = ? AND user_id = ?", id, principal.UserID).First(&bill).Error
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "bill not found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to look up bill", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toBillDetailResponse(bill))
	}
}

func updateBillHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}

		bill, ok := findOwnedBill(w, r, db, principal.UserID)
		if !ok {
			return
		}

		req, ok := decodeBillRequest(w, r)
		if !ok {
			return
		}

		bill.Name = req.Name
		bill.Amount = req.Amount
		bill.DueDay = req.DueDay
		bill.IsAutoPay = req.IsAutoPay
		bill.Notes = req.Notes

		if err := db.Save(&bill).Error; err != nil {
			http.Error(w, "failed to update bill", http.StatusInternalServerError)
			return
		}

		writeJSON(w, http.StatusOK, toBillResponse(bill))
	}
}

func deleteBillHandler(db *gorm.DB) http.HandlerFunc {
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
			var bill models.Bill
			if err := tx.Where("id = ? AND user_id = ?", id, principal.UserID).First(&bill).Error; err != nil {
				return err
			}
			if err := tx.Where("bill_id = ?", id).Delete(&models.LedgerBill{}).Error; err != nil {
				return err
			}
			return tx.Delete(&bill).Error
		})
		if errors.Is(err, gorm.ErrRecordNotFound) {
			http.Error(w, "bill not found", http.StatusNotFound)
			return
		}
		if err != nil {
			http.Error(w, "failed to delete bill", http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusNoContent)
	}
}
