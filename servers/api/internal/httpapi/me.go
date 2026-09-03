package httpapi

import (
	"encoding/json"
	"net/http"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
)

func meHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			http.Error(w, "no authenticated user", http.StatusInternalServerError)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(struct {
			ID    string `json:"id"`
			Sub   string `json:"sub"`
			Email string `json:"email"`
		}{
			ID:    principal.UserID.String(),
			Sub:   principal.Sub,
			Email: principal.Email,
		})
	}
}
