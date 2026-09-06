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

`internal/auth` defines a `Verifier` interface (`Verify(ctx, rawIDToken) (*UserPrincipal, error)`)
and a `RequireAuth` middleware wired into the router (see `cmd/api/main.go`
and `internal/httpapi/router.go`, protecting `GET /me`). `UserPrincipal`
carries `UserID` resolved (find-or-create by `sub`) against our own
database — it's never a token claim, which is why it's a separate type from
`Claims` (the raw `sub`/`email` asserted by the token).

Two `Verifier` implementations exist:
- `OIDCVerifier` — real JWT/OIDC verification. Built but unused: no IdP has
  been chosen/configured yet, and it does a live discovery call against
  `OIDC_ISSUER_URL` at construction time.
- `MockVerifier` — what's actually wired in for now. Ignores the token
  entirely and always authenticates as the seed user from
  `cmd/import-legacy-data` (`auth.SeedUserSub`/`auth.SeedUserEmail`).

Handlers read the current identity via `auth.PrincipalFromContext(ctx)`.

## Deploy

Deploy infrastructure lives at `../../platform/api/` — outside this Go
module (not nested inside it) so its `node_modules` can never collide with
Go's own tooling (an earlier revision nested it inside `servers/api/`,
which broke `go build ./...`/`go mod tidy` when CDK's bundled Go
init-templates got mistaken for real Go source), and alongside the rest of
the repo's infra-as-code (`platform/db/`, `platform/web-client/`).

`../../platform/api/Dockerfile` builds the API into a container image
(build context is `servers/api/`: `docker build -f
../../platform/api/Dockerfile -t money-bae-api:local .` run from
`servers/api/`). `../../platform/api/cdk/` is a CDK app defining the AWS
App Runner deployment (ECR repo, App Runner service, Secrets Manager
reference for `DATABASE_URL`) — see its own directory for details. Both
stacks (`MoneyBaeApiEcrStack`, `MoneyBaeApiStack`) are deployed and live
(App Runner service `ApiService8D48F45E-...`, region us-east-1). The
service's ECR source has `autoDeploymentsEnabled: false` — pushing a new
`:latest` image doesn't roll out on its own, so a deploy is: build, tag,
push to ECR, then `aws apprunner start-deployment --service-arn <arn>`
to actually roll it out.

Because `migrations.Run` (`cmd/api/main.go`) runs unconditionally on
every startup, deploying a new image immediately applies any pending
schema migration to whatever `DATABASE_URL` that Secrets Manager entry
points to — currently the shared dev Postgres, not a throwaway
per-deploy database. Treat an API deploy with the same care as a
migration against shared infrastructure.
