package httpapi

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestCorsMiddleware_SetsHeadersOnNormalRequest(t *testing.T) {
	handler := corsMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if got := rec.Header().Get("Access-Control-Allow-Origin"); got != "*" {
		t.Fatalf("expected Access-Control-Allow-Origin: *, got %q", got)
	}
	if rec.Code != http.StatusOK {
		t.Fatalf("expected the wrapped handler to still run, got %d", rec.Code)
	}
}

func TestCorsMiddleware_HandlesPreflightWithoutCallingNext(t *testing.T) {
	handler := corsMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("next handler should not be called for an OPTIONS preflight")
	}))

	req := httptest.NewRequest(http.MethodOptions, "/incomes", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if got := rec.Header().Get("Access-Control-Allow-Methods"); got == "" {
		t.Fatal("expected Access-Control-Allow-Methods to be set")
	}
}
