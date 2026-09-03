package main

import (
	"log"
	"net/http"

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

	router := httpapi.NewRouter(db)

	// Auth is built (internal/auth) but not wired in yet — enabling this
	// requires a real OIDC_ISSUER_URL/OIDC_AUDIENCE (an actual IdP chosen),
	// since NewOIDCVerifier does a live discovery call at startup and will
	// fail against a placeholder issuer. It also requires adding "context"
	// and "github.com/mrcunninghamz/money-bae/servers/api/internal/auth" to
	// the imports above.
	//
	// Wrap individual routes that need auth — NOT the whole router, which
	// would also require auth on GET /health and break App Runner's health
	// check:
	//
	// verifier, err := auth.NewOIDCVerifier(context.Background(), cfg.OIDCIssuerURL, cfg.OIDCAudience)
	// if err != nil {
	// 	log.Fatalf("failed to create OIDC verifier: %v", err)
	// }
	// mux := httpapi.NewRouter(db).(*http.ServeMux)
	// mux.Handle("GET /me", auth.RequireAuth(verifier, db)(meHandler()))
	// router = mux

	log.Printf("listening on :%s", cfg.Port)
	if err := http.ListenAndServe(":"+cfg.Port, router); err != nil {
		log.Fatal(err)
	}
}
