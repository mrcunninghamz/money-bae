package config

import (
	"os"

	"github.com/joho/godotenv"
)

type Config struct {
	DatabaseURL   string
	Port          string
	OIDCIssuerURL string
	OIDCAudience  string
}

func Load() Config {
	_ = godotenv.Load()

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	return Config{
		DatabaseURL:   os.Getenv("DATABASE_URL"),
		Port:          port,
		OIDCIssuerURL: os.Getenv("OIDC_ISSUER_URL"),
		OIDCAudience:  os.Getenv("OIDC_AUDIENCE"),
	}
}
