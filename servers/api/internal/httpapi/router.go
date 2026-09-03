package httpapi

import (
	"net/http"

	"gorm.io/gorm"
)

func NewRouter(db *gorm.DB) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", healthHandler(db))
	return mux
}
