# money-bae API

Go + GORM API backing money-bae (a personal finance app), alongside the
existing Rust TUI (`../../tui/`). See the design spec and implementation
plan for full context: `../../docs/superpowers/specs/2026-09-02-money-bae-api-design.md`.

## Build & test

```bash
go build ./...
go vet ./...
go test ./...
```

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

Deploy infrastructure lives at `../api-infrastructure/` — a sibling of this
Go module, not nested inside it, specifically so its `node_modules` can
never collide with Go's own tooling (an earlier revision nested it inside
`servers/api/`, which broke `go build ./...`/`go mod tidy` when CDK's
bundled Go init-templates got mistaken for real Go source; moving it out
fixed that at the root instead of working around it with scoped commands).

`../api-infrastructure/Dockerfile` builds the API into a container image
(build context is `servers/api/`: `docker build -f
../api-infrastructure/Dockerfile -t money-bae-api:local .` run from
`servers/api/`). `../api-infrastructure/cdk/` is a CDK app defining the AWS
App Runner deployment (ECR repo, App Runner service, Secrets Manager
reference for `DATABASE_URL`) — see its own directory for details. `cdk
deploy` has not been run; this repo only generates the template (`cdk
synth`) so far.
