# money-bae API Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the first slice of the money-bae Go API — project scaffolding, a `users` table, a health endpoint, an auth middleware shape (built but not wired in), a `platform/db` Terraform change adding a second Postgres database, and Dockerfile/CDK deploy infra for AWS App Runner.

**Architecture:** A single-module Go service (`servers/api`) using GORM over Postgres, stdlib `net/http` routing, OIDC-based auth middleware built against an interface (so it's testable without a real IdP), and a Terraform change to `platform/db` (moved from `tui/infrastructure`) to add a `money_bae_api` database alongside the existing `money_bae` on the same Azure Postgres server. Deploy path is a Dockerfile + a TypeScript CDK app targeting AWS App Runner.

**Tech Stack:** Go 1.22, `gorm.io/gorm` + `gorm.io/driver/postgres`, `github.com/google/uuid` (UUIDv7), `github.com/coreos/go-oidc/v3`, `github.com/joho/godotenv`, `github.com/glebarez/sqlite` (test-only), Terraform (azurerm provider), AWS CDK v2 (TypeScript) + `@aws-cdk/aws-apprunner-alpha`, Docker.

**Spec:** `docs/superpowers/specs/2026-09-02-money-bae-api-design.md`

## Global Constraints

- Go module path: `github.com/mrcunninghamz/money-bae/servers/api`; `go.mod` declares `go 1.27` (toolchain upgraded from 1.22 mid-plan, see Task 6's ledger entry for the version-bump follow-up commit).
- Single Go module for now — no `go.work`. Revisit only if a second Go module appears under `servers/`.
- DB-touching Go tests use `github.com/glebarez/sqlite` (pure-Go, in-memory) as a fast mock — confirmed preference, even though production targets Postgres. DSN: `file::memory:?cache=shared`.
- Auth middleware is fully built and tested but **not wired into the router** in this plan — no real IdP is configured yet.
- Terraform in `platform/db` touches **live** infrastructure (already confirmed: real Azure resources, real remote state in `stmbtfstateshared`). `terraform apply` is never run automatically by any task in this plan — only `validate`/`fmt`/`plan`. Applying is a manual, explicitly-confirmed step outside this plan's automation.
- Backend init requires `-backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"` (already fixed in the README during brainstorming).
- Health endpoint pings the DB (503 on failure, 200 `{"status":"ok"}` otherwise) — this is what App Runner's health check polls.

---

### Task 1: Move `tui/infrastructure` to `platform/db/infrastructure`

**Files:**
- Move: `tui/infrastructure/**` → `platform/db/infrastructure/**` (git mv, preserves history)

**Interfaces:** None — pure mechanical move, no code changes.

Note: nesting under `infrastructure/` (rather than directly in `platform/db/`) mirrors the convention `servers/api/infrastructure/` uses for that project's Dockerfile/CDK — every top-level project keeps its IaC in its own `infrastructure/` subfolder.

- [ ] **Step 1: Move the directory with git mv**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
mkdir -p platform/db
git mv tui/infrastructure platform/db/infrastructure
```

- [ ] **Step 2: Check for stray references to the old path**

```bash
grep -rn "tui/infrastructure" --include="*.md" --include="*.tf" --include="*.sh" . || echo "no references found"
```

Expected: `no references found`. If any hits appear, update them to `platform/db/infrastructure`.

- [ ] **Step 3: Verify the moved Terraform still inits cleanly against the live backend**

```bash
cd platform/db/infrastructure
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=core/dev.cus.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
terraform state list
```

Expected: `Terraform has been successfully initialized!` and the same 5 resources as before the move (`azurerm_resource_group.main`, `module.postgresql.azurerm_postgresql_flexible_server.main`, `module.postgresql.azurerm_postgresql_flexible_server_database.main`, and the two firewall rules).

- [ ] **Step 4: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add platform/
git commit -m "Move infrastructure Terraform to platform/db/infrastructure"
```

---

### Task 2: Add a second Postgres database via Terraform (`money_bae_api`)

**Files:**
- Modify: `platform/db/infrastructure/modules/postgresql/variables.tf`
- Modify: `platform/db/infrastructure/modules/postgresql/main.tf`
- Modify: `platform/db/infrastructure/modules/postgresql/outputs.tf`
- Modify: `platform/db/infrastructure/providers.tf`
- Modify: `platform/db/infrastructure/main.tf`
- Modify: `platform/db/infrastructure/outputs.tf`

**Interfaces:**
- Produces: module output `database_names` (`list(string)`), module output `connection_strings` (`map(string)`, sensitive) — consumed by `platform/db/infrastructure`'s own root outputs.

- [ ] **Step 1: Change the module's variable from a single name to a list**

Modify `platform/db/infrastructure/modules/postgresql/variables.tf` — replace:

```hcl
variable "database_name" {
  type        = string
  description = "Name of the database to create"
  default     = "money_bae"
}
```

with:

```hcl
variable "database_names" {
  type        = list(string)
  description = "Names of the databases to create on this server"
}
```

- [ ] **Step 2: Switch the database resource to for_each, with a moved block to protect the live database**

Modify `platform/db/infrastructure/modules/postgresql/main.tf` — replace:

```hcl
resource "azurerm_postgresql_flexible_server_database" "main" {
  name      = var.database_name
  server_id = azurerm_postgresql_flexible_server.main.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}
```

with:

```hcl
resource "azurerm_postgresql_flexible_server_database" "main" {
  for_each  = toset(var.database_names)
  name      = each.value
  server_id = azurerm_postgresql_flexible_server.main.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}

moved {
  from = azurerm_postgresql_flexible_server_database.main
  to   = azurerm_postgresql_flexible_server_database.main["money_bae"]
}
```

- [ ] **Step 3: Fix and rewrite the module's outputs for multiple databases**

Modify `platform/db/infrastructure/modules/postgresql/outputs.tf` — replace the `database_name` and `connection_string` outputs (leave `server_id`, `server_name`, `server_fqdn` untouched):

```hcl
output "database_names" {
  value       = [for db in azurerm_postgresql_flexible_server_database.main : db.name]
  description = "Names of the PostgreSQL databases"
}

output "connection_strings" {
  value = {
    for name, db in azurerm_postgresql_flexible_server_database.main :
    name => "postgresql://${var.administrator_login}:${var.administrator_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/${db.name}?sslmode=require"
  }
  description = "Map of database name to PostgreSQL connection string"
  sensitive   = true
}
```

Note: the previous `connection_string` output was malformed (an extra `${azurerm_postgresql_flexible_server.main.name}` segment and `@` instead of `:` between login and password) — this rewrite fixes it as part of the multi-database change.

- [ ] **Step 4: Pin the Terraform version the `moved` block requires**

Modify `platform/db/infrastructure/providers.tf` — add `required_version` to the existing `terraform` block:

```hcl
terraform {
  required_version = ">= 1.1"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=4.1.0"
    }
  }

  backend "azurerm" {}
}
```

- [ ] **Step 5: Update the root module to request both databases**

Modify `platform/db/infrastructure/main.tf` — in the `module "postgresql"` block, replace:

```hcl
  database_name          = "money_bae"
```

with:

```hcl
  database_names          = ["money_bae", "money_bae_api"]
```

- [ ] **Step 6: Update the root module's outputs to match**

Modify `platform/db/infrastructure/outputs.tf` — replace the `postgresql_database_name` and `postgresql_connection_string` outputs:

```hcl
output "postgresql_database_names" {
  value       = module.postgresql.database_names
  description = "Names of the PostgreSQL databases"
}

output "postgresql_connection_strings" {
  value       = module.postgresql.connection_strings
  description = "Map of database name to PostgreSQL connection string"
  sensitive   = true
}
```

- [ ] **Step 7: Validate and format**

```bash
cd platform/db/infrastructure
terraform fmt -check -recursive
terraform validate
```

Expected: `terraform fmt` reports no changes needed (or run `terraform fmt -recursive` without `-check` to auto-fix, then re-run `-check`); `terraform validate` reports `Success! The configuration is valid.`

- [ ] **Step 8: Plan, and stop for human review before any apply**

Requires `TF_VAR_money_bae_db_admin_password` set in the environment (see `platform/db/infrastructure/README.md`) — never pass the real password as a `-var` flag or write it into any file.

```bash
terraform plan -var-file="environments/dev.cus.tfvars"
```

Expected in the plan output: the existing `money_bae` database shows as **moved**, not destroyed (look for "1 to move" or the resource listed under "Terraform will perform the following actions" with a move annotation, not `-/+ destroy and then create`), and exactly **one** new database (`money_bae_api`) shows as `+ create`. If anything shows the `money_bae` database being destroyed/recreated, **stop and do not proceed** — that means the `moved` block didn't take effect as expected.

**Do not run `terraform apply` as part of this task.** Show the plan output to the user and get their explicit go-ahead before applying — this is live infrastructure.

- [ ] **Step 9: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add platform/db
git commit -m "Add money_bae_api database via platform/db/infrastructure Terraform"
```

---

### Task 3: Go module init + health endpoint stub

**Files:**
- Create: `servers/api/go.mod`
- Create: `servers/api/cmd/api/main.go`
- Create: `servers/api/internal/httpapi/router.go`
- Create: `servers/api/internal/httpapi/health.go`
- Test: `servers/api/internal/httpapi/health_test.go`

**Interfaces:**
- Produces: `httpapi.NewRouter() http.Handler` — later tasks (Task 6) change this signature to take a `*gorm.DB`.

- [ ] **Step 1: Initialize the Go module**

```bash
mkdir -p /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go mod init github.com/mrcunninghamz/money-bae/servers/api
```

- [ ] **Step 2: Write the failing test for the health handler**

Create `servers/api/internal/httpapi/health_test.go`:

```go
package httpapi

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHealthHandler_ReturnsOK(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()

	healthHandler(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}
	if body := rec.Body.String(); body != `{"status":"ok"}` {
		t.Fatalf("unexpected body: %s", body)
	}
}
```

- [ ] **Step 3: Run the test to verify it fails**

```bash
go test ./internal/httpapi/... -run TestHealthHandler_ReturnsOK -v
```

Expected: FAIL — `undefined: healthHandler` (package doesn't exist yet).

- [ ] **Step 4: Implement the health handler and router**

Create `servers/api/internal/httpapi/health.go`:

```go
package httpapi

import "net/http"

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(`{"status":"ok"}`))
}
```

Create `servers/api/internal/httpapi/router.go`:

```go
package httpapi

import "net/http"

func NewRouter() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", healthHandler)
	return mux
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
go test ./internal/httpapi/... -v
```

Expected: PASS.

- [ ] **Step 6: Wire up main.go**

Create `servers/api/cmd/api/main.go`:

```go
package main

import (
	"log"
	"net/http"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/httpapi"
)

func main() {
	router := httpapi.NewRouter()
	log.Println("listening on :8080")
	if err := http.ListenAndServe(":8080", router); err != nil {
		log.Fatal(err)
	}
}
```

- [ ] **Step 7: Verify it builds and runs**

```bash
go build ./...
go run ./cmd/api &
sleep 1
curl -s http://localhost:8080/health
kill %1
```

Expected: `curl` prints `{"status":"ok"}`.

- [ ] **Step 8: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/go.mod servers/api/cmd servers/api/internal
git commit -m "Add Go API module with health endpoint stub"
```

---

### Task 4: Config loading

**Files:**
- Create: `servers/api/internal/config/config.go`
- Test: `servers/api/internal/config/config_test.go`

**Interfaces:**
- Produces: `config.Config{DatabaseURL, Port, OIDCIssuerURL, OIDCAudience string}` and `config.Load() Config` — consumed by `cmd/api/main.go` (Task 6) and the auth adapter (Task 7).

- [ ] **Step 1: Add the godotenv dependency**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go get github.com/joho/godotenv
```

- [ ] **Step 2: Write the failing tests**

Create `servers/api/internal/config/config_test.go`:

```go
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
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
go test ./internal/config/... -v
```

Expected: FAIL — `undefined: Load`.

- [ ] **Step 4: Implement config loading**

Create `servers/api/internal/config/config.go`:

```go
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
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
go test ./internal/config/... -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/go.mod servers/api/go.sum servers/api/internal/config
git commit -m "Add config loading"
```

---

### Task 5: Data model (`Base` + `User`)

**Files:**
- Create: `servers/api/internal/models/base.go`
- Create: `servers/api/internal/models/user.go`
- Test: `servers/api/internal/models/user_test.go`

**Interfaces:**
- Produces: `models.Base` (embeds into future models — `ID uuid.UUID`, `CreatedAt`, `UpdatedAt`, `BeforeCreate` hook), `models.User{Base, Sub string, Email string}` — consumed by `internal/database` (Task 6, `AutoMigrate`) and `internal/auth` (Task 7).

- [ ] **Step 1: Add dependencies**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go get gorm.io/gorm
go get github.com/google/uuid
go get github.com/glebarez/sqlite
```

- [ ] **Step 2: Write the failing tests**

Create `servers/api/internal/models/user_test.go`:

```go
package models_test

import (
	"testing"

	"github.com/glebarez/sqlite"
	"github.com/google/uuid"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

func setupTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open("file::memory:?cache=shared"), &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	if err := db.AutoMigrate(&models.User{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func TestUser_BeforeCreate_AssignsUUIDv7WhenZero(t *testing.T) {
	db := setupTestDB(t)

	user := &models.User{Sub: "auth0|123"}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	if user.ID == uuid.Nil {
		t.Fatal("expected ID to be assigned, got zero UUID")
	}
	if user.ID.Version() != 7 {
		t.Fatalf("expected UUID version 7, got %d", user.ID.Version())
	}
}

func TestUser_BeforeCreate_PreservesProvidedID(t *testing.T) {
	db := setupTestDB(t)

	fixedID := uuid.New()
	user := &models.User{Base: models.Base{ID: fixedID}, Sub: "auth0|456"}
	if err := db.Create(user).Error; err != nil {
		t.Fatalf("failed to create user: %v", err)
	}

	if user.ID != fixedID {
		t.Fatalf("expected ID to remain %s, got %s", fixedID, user.ID)
	}
}

func TestUser_Sub_MustBeUnique(t *testing.T) {
	db := setupTestDB(t)

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err != nil {
		t.Fatalf("failed to create first user: %v", err)
	}

	if err := db.Create(&models.User{Sub: "auth0|dup"}).Error; err == nil {
		t.Fatal("expected duplicate Sub to fail, got nil error")
	}
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
go test ./internal/models/... -v
```

Expected: FAIL — `undefined: models.User` (package doesn't exist yet).

- [ ] **Step 4: Implement Base**

Create `servers/api/internal/models/base.go`:

```go
package models

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Base struct {
	ID        uuid.UUID `gorm:"type:uuid;primaryKey"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

func (b *Base) BeforeCreate(tx *gorm.DB) error {
	if b.ID == uuid.Nil {
		id, err := uuid.NewV7()
		if err != nil {
			return err
		}
		b.ID = id
	}
	return nil
}
```

- [ ] **Step 5: Implement User**

Create `servers/api/internal/models/user.go`:

```go
package models

type User struct {
	Base
	Sub   string `gorm:"uniqueIndex;not null"`
	Email string `gorm:"index"`
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
go test ./internal/models/... -v
```

Expected: PASS (all 3 tests).

- [ ] **Step 7: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/go.mod servers/api/go.sum servers/api/internal/models
git commit -m "Add Base/User models with UUIDv7 primary keys"
```

---

### Task 6: Database connection + AutoMigrate + real health check

**Files:**
- Create: `servers/api/internal/database/database.go`
- Modify: `servers/api/internal/httpapi/health.go`
- Modify: `servers/api/internal/httpapi/router.go`
- Modify: `servers/api/internal/httpapi/health_test.go` (replaces Task 3's version)
- Modify: `servers/api/cmd/api/main.go`

**Interfaces:**
- Consumes: `models.User` (Task 5).
- Produces: `database.Connect(dsn string) (*gorm.DB, error)` — consumed by `cmd/api/main.go`. `httpapi.NewRouter(db *gorm.DB) http.Handler` — signature change from Task 3, consumed by `cmd/api/main.go` and (in spirit, commented out) Task 7's auth wiring.

- [ ] **Step 1: Add the Postgres driver dependency**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go get gorm.io/driver/postgres
```

- [ ] **Step 2: Implement the database connection wrapper**

Create `servers/api/internal/database/database.go`:

```go
package database

import (
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func Connect(dsn string) (*gorm.DB, error) {
	return gorm.Open(postgres.Open(dsn), &gorm.Config{})
}
```

No dedicated unit test for this function: it's a two-line pass-through to `gorm.Open` with the Postgres driver, and there's nothing to sqlite-mock here since the whole point is the Postgres-specific driver wiring. It's verified manually in Task 8's end-to-end check against a real local Postgres.

- [ ] **Step 3: Rewrite the health handler test for DB-ping behavior (replaces Task 3's test)**

Replace the contents of `servers/api/internal/httpapi/health_test.go`:

```go
package httpapi

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
)

func openTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open("file::memory:?cache=shared"), &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	return db
}

func TestHealthHandler_ReturnsOKWhenDBReachable(t *testing.T) {
	db := openTestDB(t)
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
	db := openTestDB(t)
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
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
go test ./internal/httpapi/... -v
```

Expected: FAIL — compile error, `healthHandler` doesn't take a `*gorm.DB` argument yet.

- [ ] **Step 5: Update the health handler and router to take a `*gorm.DB`**

Replace the contents of `servers/api/internal/httpapi/health.go`:

```go
package httpapi

import (
	"net/http"

	"gorm.io/gorm"
)

func healthHandler(db *gorm.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		sqlDB, err := db.DB()
		if err != nil {
			writeHealthResponse(w, http.StatusServiceUnavailable)
			return
		}
		if err := sqlDB.PingContext(r.Context()); err != nil {
			writeHealthResponse(w, http.StatusServiceUnavailable)
			return
		}
		writeHealthResponse(w, http.StatusOK)
	}
}

func writeHealthResponse(w http.ResponseWriter, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if status == http.StatusOK {
		w.Write([]byte(`{"status":"ok"}`))
	} else {
		w.Write([]byte(`{"status":"unavailable"}`))
	}
}
```

Replace the contents of `servers/api/internal/httpapi/router.go`:

```go
package httpapi

import (
	"net/http"

	"gorm.io/gorm"
)

func NewRouter(db *gorm.DB) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /health", healthHandler(db))
	return mux
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
go test ./internal/httpapi/... -v
```

Expected: PASS (both tests).

- [ ] **Step 7: Wire the database into main.go**

Replace the contents of `servers/api/cmd/api/main.go`:

```go
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

	log.Printf("listening on :%s", cfg.Port)
	if err := http.ListenAndServe(":"+cfg.Port, router); err != nil {
		log.Fatal(err)
	}
}
```

- [ ] **Step 8: Verify it builds**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go build ./...
```

Expected: no errors. (Running it end-to-end against a real Postgres happens in Task 8, once env files exist.)

- [ ] **Step 9: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/go.mod servers/api/go.sum servers/api/internal/database servers/api/internal/httpapi servers/api/cmd
git commit -m "Wire database connection and real health check"
```

---

### Task 7: Auth middleware (built, not wired in)

**Files:**
- Create: `servers/api/internal/auth/auth.go`
- Create: `servers/api/internal/auth/middleware.go`
- Create: `servers/api/internal/auth/oidc_verifier.go`
- Test: `servers/api/internal/auth/middleware_test.go`

**Interfaces:**
- Consumes: `models.User` (Task 5), `config.Config` (Task 4, for `OIDCIssuerURL`/`OIDCAudience` — used only by the production adapter, not by tests).
- Produces: `auth.Claims{Sub, Email string}`, `auth.Verifier` interface (`Verify(ctx, rawIDToken) (*Claims, error)`), `auth.RequireAuth(verifier Verifier, db *gorm.DB) func(http.Handler) http.Handler`, `auth.UserFromContext(ctx) (*models.User, bool)`, `auth.NewOIDCVerifier(ctx, issuerURL, audience string) (*OIDCVerifier, error)` (production `Verifier` implementation — not called anywhere in this plan, wired in commented-out form in Task 8).

- [ ] **Step 1: Add the go-oidc dependency**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go get github.com/coreos/go-oidc/v3
```

- [ ] **Step 2: Write the failing tests**

Create `servers/api/internal/auth/middleware_test.go`:

```go
package auth_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/auth"
	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type fakeVerifier struct {
	claims *auth.Claims
	err    error
}

func (f *fakeVerifier) Verify(ctx context.Context, rawIDToken string) (*auth.Claims, error) {
	return f.claims, f.err
}

func setupTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	db, err := gorm.Open(sqlite.Open("file::memory:?cache=shared"), &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to open test db: %v", err)
	}
	if err := db.AutoMigrate(&models.User{}); err != nil {
		t.Fatalf("failed to migrate: %v", err)
	}
	return db
}

func TestRequireAuth_MissingHeader_Returns401(t *testing.T) {
	db := setupTestDB(t)
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
	db := setupTestDB(t)
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
	db := setupTestDB(t)
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
	db := setupTestDB(t)
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
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
go test ./internal/auth/... -v
```

Expected: FAIL — package `auth` doesn't exist yet.

- [ ] **Step 4: Implement Claims and the Verifier interface**

Create `servers/api/internal/auth/auth.go`:

```go
package auth

import "context"

type Claims struct {
	Sub   string
	Email string
}

type Verifier interface {
	Verify(ctx context.Context, rawIDToken string) (*Claims, error)
}
```

- [ ] **Step 5: Implement the middleware**

Create `servers/api/internal/auth/middleware.go`:

```go
package auth

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"gorm.io/gorm"

	"github.com/mrcunninghamz/money-bae/servers/api/internal/models"
)

type contextKey string

const userContextKey contextKey = "auth_user"

func RequireAuth(verifier Verifier, db *gorm.DB) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") {
				http.Error(w, "missing bearer token", http.StatusUnauthorized)
				return
			}
			rawToken := strings.TrimPrefix(authHeader, "Bearer ")

			claims, err := verifier.Verify(r.Context(), rawToken)
			if err != nil {
				http.Error(w, "invalid token", http.StatusUnauthorized)
				return
			}

			var user models.User
			err = db.Where("sub = ?", claims.Sub).First(&user).Error
			switch {
			case errors.Is(err, gorm.ErrRecordNotFound):
				user = models.User{Sub: claims.Sub, Email: claims.Email}
				if err := db.Create(&user).Error; err != nil {
					http.Error(w, "failed to provision user", http.StatusInternalServerError)
					return
				}
			case err != nil:
				http.Error(w, "failed to look up user", http.StatusInternalServerError)
				return
			}

			ctx := context.WithValue(r.Context(), userContextKey, &user)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

func UserFromContext(ctx context.Context) (*models.User, bool) {
	user, ok := ctx.Value(userContextKey).(*models.User)
	return user, ok
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
go test ./internal/auth/... -v
```

Expected: PASS (all 4 tests).

- [ ] **Step 7: Implement the production OIDC adapter (not called by any test — wired in commented-out form in Task 8)**

Create `servers/api/internal/auth/oidc_verifier.go`:

```go
package auth

import (
	"context"

	"github.com/coreos/go-oidc/v3/oidc"
)

type OIDCVerifier struct {
	verifier *oidc.IDTokenVerifier
}

func NewOIDCVerifier(ctx context.Context, issuerURL, audience string) (*OIDCVerifier, error) {
	provider, err := oidc.NewProvider(ctx, issuerURL)
	if err != nil {
		return nil, err
	}
	return &OIDCVerifier{
		verifier: provider.Verifier(&oidc.Config{ClientID: audience}),
	}, nil
}

func (v *OIDCVerifier) Verify(ctx context.Context, rawIDToken string) (*Claims, error) {
	idToken, err := v.verifier.Verify(ctx, rawIDToken)
	if err != nil {
		return nil, err
	}

	var claims struct {
		Sub   string `json:"sub"`
		Email string `json:"email"`
	}
	if err := idToken.Claims(&claims); err != nil {
		return nil, err
	}

	return &Claims{Sub: claims.Sub, Email: claims.Email}, nil
}
```

- [ ] **Step 8: Verify the whole module still builds**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go build ./...
```

Expected: no errors (this file compiles even though nothing calls `NewOIDCVerifier` yet).

- [ ] **Step 9: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/go.mod servers/api/go.sum servers/api/internal/auth
git commit -m "Add auth middleware (built, not wired into router)"
```

---

### Task 8: End-to-end wiring, env files, and local verification

**Files:**
- Create: `servers/api/.env.local.example`
- Create: `servers/api/.env.dev.example`
- Create: `servers/api/use-local-env.sh`
- Create: `servers/api/use-dev-env.sh`
- Modify: `servers/api/cmd/api/main.go`
- Modify: `.gitignore` (repo root)

**Interfaces:** None new — this task wires existing pieces together and adds local dev conventions.

- [ ] **Step 1: Add the local env-file templates**

Create `servers/api/.env.local.example`:

```
# Copy to .env.local and point at whatever Postgres you're using locally
# (a local install, or the shared Azure dev database — your choice), then
# run ./use-local-env.sh to activate it.
DATABASE_URL=postgres://username:password@localhost:5432/money_bae_api
PORT=8080
```

Create `servers/api/.env.dev.example`:

```
# Copy to .env.dev to point at the shared Azure dev database, then run
# ./use-dev-env.sh to activate it.
DATABASE_URL=postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae_api?sslmode=require
PORT=8080
```

- [ ] **Step 2: Add the switch scripts**

Create `servers/api/use-local-env.sh`:

```bash
#!/bin/bash
cp .env.local .env
echo "✓ Switched to local environment"
```

Create `servers/api/use-dev-env.sh`:

```bash
#!/bin/bash
cp .env.dev .env
echo "✓ Switched to dev environment"
```

```bash
chmod +x /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api/use-local-env.sh
chmod +x /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api/use-dev-env.sh
```

- [ ] **Step 3: Add `.env.local` to the repo's .gitignore**

Modify `.gitignore` (repo root) — add a line next to the existing `.env.dev`/`.env.prod` entries:

```
.env
.env.dev
.env.prod
.env.local
```

- [ ] **Step 4: Add the commented-out auth wiring to main.go**

Modify `servers/api/cmd/api/main.go` — insert this comment block between `router := httpapi.NewRouter(db)` and the `log.Printf("listening on...")` line:

```go
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
```

- [ ] **Step 5: Verify the module still builds with the new comment block**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go build ./...
go vet ./...
```

Expected: no errors.

- [ ] **Step 6: Manually verify end-to-end against a real local Postgres**

```bash
createdb money_bae_api
cp .env.local.example .env.local
# edit .env.local: set DATABASE_URL=postgres://<your-local-user>@localhost:5432/money_bae_api
./use-local-env.sh
go run ./cmd/api &
sleep 1
curl -s http://localhost:8080/health
kill %1
```

Expected: `curl` prints `{"status":"ok"}`, and `psql money_bae_api -c '\dt'` shows a `users` table (created by `AutoMigrate`).

- [ ] **Step 7: Run the full test suite one more time**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
go test ./... -v
```

Expected: PASS across all packages.

- [ ] **Step 8: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/.env.local.example servers/api/.env.dev.example servers/api/use-local-env.sh servers/api/use-dev-env.sh servers/api/cmd .gitignore
git commit -m "Wire main.go end-to-end, add local env-file conventions"
```

---

### Task 9: Dockerfile

**Files:**
- Create: `servers/api/infrastructure/Dockerfile`
- Create: `servers/api/.dockerignore`

**Interfaces:** None — build artifact only.

- [ ] **Step 1: Write the Dockerfile**

Create `servers/api/infrastructure/Dockerfile`:

```dockerfile
FROM golang:1.27.1-alpine AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /app/server ./cmd/api

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/server /server
ENTRYPOINT ["/server"]
```

- [ ] **Step 2: Exclude irrelevant/sensitive content from the build context**

Since the build context is `servers/api/` (the whole directory, via `COPY . .`), without a `.dockerignore` it would also send the CDK app's `node_modules`, and any local `.env`/`.env.local` files with real credentials, to the Docker daemon.

Create `servers/api/.dockerignore`:

```
infrastructure/
.env
.env.local
.env.dev
*.example
```

- [ ] **Step 3: Verify it builds locally**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api
docker build -f infrastructure/Dockerfile -t money-bae-api:local .
```

Expected: build completes successfully (note: `go.sum` must exist — it will, from Tasks 4-7's `go get` calls).

- [ ] **Step 4: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/infrastructure/Dockerfile servers/api/.dockerignore
git commit -m "Add API Dockerfile"
```

---

### Task 10: CDK app (ECR + App Runner)

**Files:**
- Create: `servers/api/infrastructure/cdk/.gitignore`
- Create: `servers/api/infrastructure/cdk/package.json`
- Create: `servers/api/infrastructure/cdk/tsconfig.json`
- Create: `servers/api/infrastructure/cdk/cdk.json`
- Create: `servers/api/infrastructure/cdk/bin/api.ts`
- Create: `servers/api/infrastructure/cdk/lib/api-stack.ts`
- Test: `servers/api/infrastructure/cdk/lib/api-stack.test.ts`

**Interfaces:** None — self-contained CDK app, no other task depends on its exports.

- [ ] **Step 1: Ignore CDK/npm build output**

Create `servers/api/infrastructure/cdk/.gitignore`:

```
node_modules/
cdk.out/
*.js
*.d.ts
```

- [ ] **Step 2: Write package.json**

Create `servers/api/infrastructure/cdk/package.json`:

```json
{
  "name": "money-bae-api-infra",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "cdk": "cdk"
  },
  "devDependencies": {
    "@types/jest": "^29.5.12",
    "@types/node": "^20.11.0",
    "aws-cdk": "^2.150.0",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.2",
    "ts-node": "^10.9.2",
    "typescript": "^5.4.5"
  },
  "dependencies": {
    "@aws-cdk/aws-apprunner-alpha": "^2.150.0-alpha.0",
    "aws-cdk-lib": "^2.150.0",
    "constructs": "^10.3.0",
    "source-map-support": "^0.5.21"
  },
  "jest": {
    "preset": "ts-jest",
    "testEnvironment": "node"
  }
}
```

- [ ] **Step 3: Write tsconfig.json**

Create `servers/api/infrastructure/cdk/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "declaration": true,
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noImplicitThis": true,
    "alwaysStrict": true,
    "noImplicitReturns": true,
    "inlineSourceMap": true,
    "inlineSources": true,
    "experimentalDecorators": true,
    "strictPropertyInitialization": false,
    "typeRoots": ["./node_modules/@types"]
  },
  "exclude": ["node_modules", "cdk.out"]
}
```

- [ ] **Step 4: Write cdk.json**

Create `servers/api/infrastructure/cdk/cdk.json`:

```json
{
  "app": "npx ts-node --prefer-ts-exts bin/api.ts",
  "watch": {
    "include": ["**"],
    "exclude": ["README.md", "cdk*.json", "**/*.d.ts", "**/*.js", "node_modules", "cdk.out"]
  }
}
```

- [ ] **Step 5: Write the failing test**

Create `servers/api/infrastructure/cdk/lib/api-stack.test.ts`:

```ts
import { App } from 'aws-cdk-lib';
import { Template } from 'aws-cdk-lib/assertions';
import { ApiStack } from './api-stack';

test('creates an ECR repository named money-bae-api', () => {
  const app = new App();
  const stack = new ApiStack(app, 'TestStack');
  const template = Template.fromStack(stack);

  template.hasResourceProperties('AWS::ECR::Repository', {
    RepositoryName: 'money-bae-api',
  });
});

test('creates an App Runner service with a health check on /health', () => {
  const app = new App();
  const stack = new ApiStack(app, 'TestStack');
  const template = Template.fromStack(stack);

  template.hasResourceProperties('AWS::AppRunner::Service', {
    HealthCheckConfiguration: {
      Path: '/health',
    },
  });
});
```

- [ ] **Step 6: Install dependencies and run the test to verify it fails**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api/infrastructure/cdk
npm install
npm test
```

Expected: FAIL — `Cannot find module './api-stack'` (doesn't exist yet).

- [ ] **Step 7: Implement the stack**

Create `servers/api/infrastructure/cdk/lib/api-stack.ts`:

```ts
import * as cdk from 'aws-cdk-lib';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';
import * as apprunner from '@aws-cdk/aws-apprunner-alpha';
import { Construct } from 'constructs';

export class ApiStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const repo = new ecr.Repository(this, 'ApiRepo', { repositoryName: 'money-bae-api' });
    const dbSecret = secretsmanager.Secret.fromSecretNameV2(this, 'DbSecret', 'money-bae-api/database-url');

    new apprunner.Service(this, 'ApiService', {
      source: apprunner.Source.fromEcr({
        repository: repo,
        tagOrDigest: 'latest',
        imageConfiguration: {
          environmentSecrets: { DATABASE_URL: apprunner.Secret.fromSecretsManager(dbSecret) },
        },
      }),
      healthCheck: apprunner.HealthCheck.http({ path: '/health' }),
    });
  }
}
```

- [ ] **Step 8: Run the test to verify it passes**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api/infrastructure/cdk
npm test
```

Expected: PASS (both tests).

- [ ] **Step 9: Write the app entry point**

Create `servers/api/infrastructure/cdk/bin/api.ts`:

```ts
#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { ApiStack } from '../lib/api-stack';

const app = new cdk.App();
new ApiStack(app, 'MoneyBaeApiStack', {
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION,
  },
});
```

- [ ] **Step 10: Verify the app synthesizes**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae/servers/api/infrastructure/cdk
npx cdk synth
```

Expected: prints a CloudFormation template with no errors. This does **not** touch AWS — `synth` is local template generation only. `cdk deploy` (and the manual secret-creation/image-push bootstrap steps documented in the spec) are explicitly out of scope for this plan.

- [ ] **Step 11: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/api/infrastructure/cdk
git status
```

Confirm the output does **not** list anything under `node_modules/` or `cdk.out/`, nor any compiled `.js`/`.d.ts` files next to the `.ts` sources — the `.gitignore` from Step 1 should have excluded them. Then:

```bash
git commit -m "Add CDK app for API Gateway infra (ECR + App Runner)"
```

---

### Task 11: Dedicated local-dev Postgres via docker-compose

**Files:**
- Create: `servers/local-dev/docker-compose.yml`
- Create: `servers/local-dev/README.md`
- Modify: `servers/api/.env.local.example`

**Interfaces:** None — infra-only, no Go code touched.

Added mid-plan: local dev had been using another project's Docker Postgres container (`fern-pg`) as a stopgap during Task 8's manual verification. This task gives money-bae its own dedicated local Postgres, independent of any other project's container lifecycle, on host port 5433 (5432 is already taken by `fern-pg` on this machine).

**Incident during this task**: the first version of this compose file had no explicit `name:` field. Docker Compose derives a project's name from its directory's basename when none is set — and `servers/local-dev/` collided with an unrelated project's compose file that *also* lives in a directory named `local-dev`. Both files defined a service literally named `postgres`, so Compose treated the new container as an update to the existing one and replaced (deleted) the other project's live `fern-pg` container. The data was recovered from an orphaned volume and that project's own compose file was fixed in its own repo (separate PR, out of scope here) — but the lesson applies here too: **always set an explicit top-level `name:`** so the project's identity never depends on directory-naming coincidences with other repos on the same machine.

- [ ] **Step 1: Write the compose file**

Create `servers/local-dev/docker-compose.yml`:

```yaml
name: money-bae
services:
  postgres:
    image: postgres:16
    container_name: money-bae-pg
    ports:
      - "5433:5432"
    environment:
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: root
      POSTGRES_DB: money_bae_api
    volumes:
      - money-bae-local-dev-data:/var/lib/postgresql/data

volumes:
  money-bae-local-dev-data:
```

`POSTGRES_DB: money_bae_api` makes the official Postgres image create that database automatically on first boot — no manual `createdb` step needed. The top-level `name: money-bae` is the fix described above — it pins the Compose project identity regardless of what directory this file happens to live in.

- [ ] **Step 2: Write a short README**

Create `servers/local-dev/README.md`:

```markdown
# Local Dev Postgres

Dedicated Postgres for local money-bae API development — independent of any
other project's containers.

## Start

```bash
docker compose up -d
```

Creates a `money_bae_api` database automatically (via `POSTGRES_DB`), reachable at
`localhost:5433`, credentials `admin`/`root`. Data persists in a named Docker
volume (`money-bae-local-dev-data`) across restarts.

## Stop

```bash
docker compose down
```

Add `-v` to also delete the data volume (starts fresh next time).

## Connection string

```
postgres://admin:root@localhost:5433/money_bae_api?sslmode=disable
```

Matches `servers/api/.env.local.example`.
```

- [ ] **Step 3: Point the API's local env template at it**

Modify `servers/api/.env.local.example` — replace its `DATABASE_URL` line:

```
DATABASE_URL=postgres://admin:root@localhost:5433/money_bae_api?sslmode=disable
```

(Port `5433`, matching this compose file — not `5432`, which is taken by an unrelated project's container on this machine.)

- [ ] **Step 4: Verify it actually works**

```bash
cd servers/local-dev
docker compose up -d
docker compose ps
psql "postgres://admin:root@localhost:5433/money_bae_api?sslmode=disable" -c '\conninfo' 2>&1 || \
  docker exec money-bae-pg psql -U admin -d money_bae_api -c '\conninfo'
```

Expected: container `money-bae-pg` is `Up`, and the connection succeeds (via `psql` directly if installed, or `docker exec` into the container if not).

- [ ] **Step 5: Commit**

```bash
cd /Users/kmerecido/Documents/Projects/mrcunninghamz/money-bae
git add servers/local-dev servers/api/.env.local.example
git commit -m "Add dedicated local-dev Postgres via docker-compose"
```
