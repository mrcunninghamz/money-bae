package httpapi

import (
	"net/http"

	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
)

func NewRouter(db *gorm.DB, verifier auth.Verifier) http.Handler {
	requireAuth := auth.RequireAuth(verifier)

	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", healthHandler(db))
	mux.Handle("GET /me", requireAuth(meHandler()))
	mux.Handle("POST /incomes", requireAuth(createIncomeHandler(db)))
	mux.Handle("GET /incomes", requireAuth(listIncomesHandler(db)))
	mux.Handle("GET /incomes/{id}", requireAuth(getIncomeHandler(db)))
	mux.Handle("PUT /incomes/{id}", requireAuth(updateIncomeHandler(db)))
	mux.Handle("DELETE /incomes/{id}", requireAuth(deleteIncomeHandler(db)))
	return loggingMiddleware(mux)
}
