package main

import (
	"log"
	"net/http"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/config"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/database"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/httpapi"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

func main() {
	cfg := config.Load()

	db, err := database.Connect(cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("failed to connect to database: %v", err)
	}
	if err := db.AutoMigrate(&models.User{}); err != nil {
		log.Fatalf("failed to migrate database: %v", err)
	}

	router := httpapi.NewRouter(db)

	// Auth is built (internal/auth) but not wired in yet — enabling this
	// requires a real OIDC_ISSUER_URL/OIDC_AUDIENCE (an actual IdP chosen),
	// since NewOIDCVerifier does a live discovery call at startup and will
	// fail against a placeholder issuer.
	//
	// verifier, err := auth.NewOIDCVerifier(context.Background(), cfg.OIDCIssuerURL, cfg.OIDCAudience)
	// if err != nil {
	// 	log.Fatalf("failed to create OIDC verifier: %v", err)
	// }
	// router = auth.RequireAuth(verifier, db)(router) // wrap whichever routes need auth

	log.Printf("listening on :%s", cfg.Port)
	if err := http.ListenAndServe(":"+cfg.Port, router); err != nil {
		log.Fatal(err)
	}
}
