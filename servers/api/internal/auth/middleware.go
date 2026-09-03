package auth

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type contextKey string

const userContextKey contextKey = "auth_user"

func RequireAuth(verifier Verifier, db *gorm.DB) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") {
				http.Error(w, "missing bearer token", http.StatusUnauthorized)
				return
			}
			rawToken := strings.TrimPrefix(authHeader, "Bearer ")

			claims, err := verifier.Verify(r.Context(), rawToken)
			if err != nil {
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}

			var user models.User
			err = db.Where("sub = ?", claims.Sub).First(&user).Error
			switch {
			case errors.Is(err, gorm.ErrRecordNotFound):
				user = models.User{Sub: claims.Sub, Email: claims.Email}
				if err := db.Create(&user).Error; err != nil {
					http.Error(w, "failed to provision user", http.StatusInternalServerError)
					return
				}
			case err != nil:
				http.Error(w, "failed to look up user", http.StatusInternalServerError)
				return
			}

			ctx := context.WithValue(r.Context(), userContextKey, &user)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func UserFromContext(ctx context.Context) (*models.User, bool) {
	user, ok := ctx.Value(userContextKey).(*models.User)
	return user, ok
}
