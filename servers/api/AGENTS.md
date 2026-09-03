# money-bae API

Go + GORM API backing money-bae (a personal finance app), alongside the
existing Rust TUI (`../../tui/`). See the design spec and implementation
plan for full context: `../../docs/superpowers/specs/2026-09-02-money-bae-api-design.md`.

## Build & test

**Always scope Go commands to `./cmd/... ./internal/...`, never bare `./...`:**

```bash
go build ./cmd/... ./internal/...
go vet ./cmd/... ./internal/...
go test ./cmd/... ./internal/...
```

`infrastructure/cdk/` is a separate TypeScript/CDK project nested inside this
Go module's directory tree. Once you run `npm install` there, its
`node_modules/` contains files (from the `aws-cdk` package's Go init
templates) that aren't valid Go source and will break a bare `go build ./...`
or `go test ./...` with a cryptic `invalid input file name` error. Scoping to
`./cmd/... ./internal/...` avoids this entirely.

## Local development

```bash
cp .env.local.example .env.local   # edit DATABASE_URL for your local setup
./use-local-env.sh                  # or ./use-dev-env.sh for the shared dev DB
go run ./cmd/api
```

A dedicated local Postgres is available via `../local-dev/` (docker-compose,
port 5433) — see `../local-dev/README.md`.

## Auth

`internal/auth` implements OIDC-based auth middleware (verify a JWT,
find-or-create the user by `sub`), but it is **not wired into the router
yet** — no IdP has been chosen/configured. See `cmd/api/main.go` for the
commented-out wiring shape.

## Deploy

`infrastructure/Dockerfile` builds the API into a container image.
`infrastructure/cdk/` is a CDK app defining the AWS App Runner deployment
(ECR repo, App Runner service, Secrets Manager reference for
`DATABASE_URL`) — see its own directory for details. `cdk deploy` has not
been run; this repo only generates the template (`cdk synth`) so far.
