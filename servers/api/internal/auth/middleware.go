package auth

import (
	"context"
	"net/http"
	"strings"
)

type contextKey string

const principalContextKey contextKey = "auth_principal"

func RequireAuth(verifier Verifier) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") {
				http.Error(w, "missing bearer token", http.StatusUnauthorized)
				return
			}
			rawToken := strings.TrimPrefix(authHeader, "Bearer ")

			principal, err := verifier.Verify(r.Context(), rawToken)
			if err != nil {
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}

			ctx := context.WithValue(r.Context(), principalContextKey, principal)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func PrincipalFromContext(ctx context.Context) (*UserPrincipal, bool) {
	principal, ok := ctx.Value(principalContextKey).(*UserPrincipal)
	return principal, ok
}
