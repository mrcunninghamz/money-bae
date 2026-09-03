package httpapi

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/testdb"
)

func TestHealthHandler_ReturnsOKWhenDBReachable(t *testing.T) {
	db := testdb.New(t)
	handler := healthHandler(db)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	handler(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}
	if body := rec.Body.String(); body != `{"status":"ok"}` {
		t.Fatalf("unexpected body: %s", body)
	}
}

func TestHealthHandler_Returns503WhenDBUnreachable(t *testing.T) {
	db := testdb.New(t)
	sqlDB, err := db.DB()
	if err != nil {
		t.Fatalf("failed to get underlying sql.DB: %v", err)
	}
	sqlDB.Close()

	handler := healthHandler(db)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	handler(rec, req)

	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("expected status 503, got %d", rec.Code)
	}
}
