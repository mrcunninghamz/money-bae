package auth_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

type fakeVerifier struct {
	claims *auth.Claims
	err    error
}

func (f *fakeVerifier) Verify(ctx context.Context, rawIDToken string) (*auth.Claims, error) {
	return f.claims, f.err
}

func TestRequireAuth_MissingHeader_Returns401(t *testing.T) {
	db := testdb.New(t, &models.User{})
	handler := auth.RequireAuth(&fakeVerifier{}, db)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
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
	db := testdb.New(t, &models.User{})
	handler := auth.RequireAuth(&fakeVerifier{err: errors.New("bad token")}, db)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
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

func TestRequireAuth_NewSub_CreatesUser(t *testing.T) {
	db := testdb.New(t, &models.User{})
	var gotUser *models.User
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user, ok := auth.UserFromContext(r.Context())
		if !ok {
			t.Fatal("expected user in context")
		}
		gotUser = user
		w.WriteHeader(http.StatusOK)
	})

	handler := auth.RequireAuth(&fakeVerifier{claims: &auth.Claims{Sub: "auth0|new", Email: "new@example.com"}}, db)(next)

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	req.Header.Set("Authorization", "Bearer validtoken")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if gotUser == nil || gotUser.Sub != "auth0|new" {
		t.Fatalf("expected user with sub auth0|new, got %+v", gotUser)
	}

	var count int64
	db.Model(&models.User{}).Where("sub = ?", "auth0|new").Count(&count)
	if count != 1 {
		t.Fatalf("expected exactly 1 user row, got %d", count)
	}
}

func TestRequireAuth_ExistingSub_ReusesUser(t *testing.T) {
	db := testdb.New(t, &models.User{})
	existing := models.User{Sub: "auth0|existing", Email: "existing@example.com"}
	if err := db.Create(&existing).Error; err != nil {
		t.Fatalf("failed to seed user: %v", err)
	}

	var gotUser *models.User
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user, _ := auth.UserFromContext(r.Context())
		gotUser = user
		w.WriteHeader(http.StatusOK)
	})

	handler := auth.RequireAuth(&fakeVerifier{claims: &auth.Claims{Sub: "auth0|existing"}}, db)(next)

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	req.Header.Set("Authorization", "Bearer validtoken")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if gotUser == nil || gotUser.ID != existing.ID {
		t.Fatalf("expected to reuse existing user %s, got %+v", existing.ID, gotUser)
	}

	var count int64
	db.Model(&models.User{}).Where("sub = ?", "auth0|existing").Count(&count)
	if count != 1 {
		t.Fatalf("expected exactly 1 user row, got %d", count)
	}
}
