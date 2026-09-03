package httpapi

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func TestMeEndpoint_RequiresAuth(t *testing.T) {
	db := testdb.New(t, &models.User{})
	router := NewRouter(db, auth.MockVerifier{DB: db})

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 without a bearer token, got %d", rec.Code)
	}
}

func TestMeEndpoint_ReturnsSeedUserClaims(t *testing.T) {
	db := testdb.New(t, &models.User{})
	router := NewRouter(db, auth.MockVerifier{DB: db})

	req := httptest.NewRequest(http.MethodGet, "/me", nil)
	req.Header.Set("Authorization", "Bearer anything")
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var got struct {
		Sub   string `json:"sub"`
		Email string `json:"email"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if got.Sub != auth.SeedUserSub || got.Email != auth.SeedUserEmail {
		t.Fatalf("expected seed user claims, got %+v", got)
	}
}
