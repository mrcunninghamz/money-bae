package auth_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
)

type fakeVerifier struct {
	principal *auth.UserPrincipal
	err       error
}

func (f *fakeVerifier) Verify(ctx context.Context, rawIDToken string) (*auth.UserPrincipal, error) {
	return f.principal, f.err
}

func TestRequireAuth_MissingHeader_Returns401(t *testing.T) {
	handler := auth.RequireAuth(&fakeVerifier{})(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("next handler should not be called")
	}))

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestRequireAuth_VerifierError_Returns401(t *testing.T) {
	handler := auth.RequireAuth(&fakeVerifier{err: errors.New("bad token")})(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("next handler should not be called")
	}))

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	req.Header.Set("Authorization", "Bearer badtoken")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestRequireAuth_ValidToken_PutsResolvedPrincipalInContext(t *testing.T) {
	want := &auth.UserPrincipal{UserID: uuid.New(), Sub: "auth0|existing", Email: "existing@example.com"}

	var got *auth.UserPrincipal
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		principal, ok := auth.PrincipalFromContext(r.Context())
		if !ok {
			t.Fatal("expected principal in context")
		}
		got = principal
		w.WriteHeader(http.StatusOK)
	})

	handler := auth.RequireAuth(&fakeVerifier{principal: want})(next)

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	req.Header.Set("Authorization", "Bearer validtoken")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if got == nil || *got != *want {
		t.Fatalf("expected principal %+v in context, got %+v", want, got)
	}
}
