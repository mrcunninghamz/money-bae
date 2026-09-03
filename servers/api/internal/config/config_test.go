package config

import "testing"

func TestLoad_DefaultsPortTo8080(t *testing.T) {
	t.Setenv("PORT", "")
	cfg := Load()
	if cfg.Port != "8080" {
		t.Fatalf("expected default port 8080, got %q", cfg.Port)
	}
}

func TestLoad_ReadsPortFromEnv(t *testing.T) {
	t.Setenv("PORT", "9090")
	cfg := Load()
	if cfg.Port != "9090" {
		t.Fatalf("expected port 9090, got %q", cfg.Port)
	}
}

func TestLoad_ReadsDatabaseURLFromEnv(t *testing.T) {
	t.Setenv("DATABASE_URL", "postgres://example")
	cfg := Load()
	if cfg.DatabaseURL != "postgres://example" {
		t.Fatalf("expected DatabaseURL to be set, got %q", cfg.DatabaseURL)
	}
}
