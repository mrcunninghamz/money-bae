package main

import (
	"log"
	"net/http"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/config"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/database"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/httpapi"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/migrations"
)

func main() {
	cfg := config.Load()

	db, err := database.Connect(cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("failed to connect to database: %v", err)
	}
	if err := migrations.Run(db); err != nil {
		log.Fatalf("failed to migrate database: %v", err)
	}

	// MockVerifier stands in for OIDCVerifier until a real IdP is chosen —
	// it always authenticates as the seed user from cmd/import-legacy-data.
	router := httpapi.NewRouter(db, auth.MockVerifier{DB: db})

	log.Printf("listening on :%s", cfg.Port)
	if err := http.ListenAndServe(":"+cfg.Port, router); err != nil {
		log.Fatal(err)
	}
}
