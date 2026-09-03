package httpapi

import (
	"net/http"

	"gorm.io/gorm"
)

func healthHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		sqlDB, err := db.DB()
		if err != nil {
			writeHealthResponse(w, http.StatusServiceUnavailable)
			return
		}
		if err := sqlDB.PingContext(r.Context()); err != nil {
			writeHealthResponse(w, http.StatusServiceUnavailable)
			return
		}
		writeHealthResponse(w, http.StatusOK)
	}
}

func writeHealthResponse(w http.ResponseWriter, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if status == http.StatusOK {
		w.Write([]byte(`{"status":"ok"}`))
	} else {
		w.Write([]byte(`{"status":"unavailable"}`))
	}
}
