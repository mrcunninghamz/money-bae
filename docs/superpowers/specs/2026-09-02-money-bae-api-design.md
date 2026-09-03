# money-bae API — Design Spec

Date: 2026-09-02
Status: Approved design, pre-implementation

## Purpose

Build a Go API (GORM ORM) that will eventually back a web/mobile frontend for
money-bae, alongside the existing Rust TUI. This is the first sub-project of
a larger split; two related sub-projects are explicitly deferred (see
"Deferred / out of scope").

## Scope of this pass

**In scope**: project scaffolding only — Go module, GORM+Postgres connection,
a `users` table, a health endpoint, the auth middleware *shape* (built but not
wired in), local dev env-file conventions, the `platform/db` Terraform
change to add a second database, and the AWS deployment infra (Dockerfile +
CDK) for that scaffold.

**Out of scope** (deferred, own future spec/plan each):
- Full CRUD for domain entities (bills, incomes, ledgers, ledger_bills,
  ptos, pto_plan, holiday_hours) — nothing beyond `users` is modeled yet.
- Choosing the actual IdP (Auth0 vs. Microsoft Entra External ID). The auth
  middleware is built to be provider-agnostic (plain OIDC) but is not wired
  into the router until a real issuer is chosen.
- CI/CD pipeline (image build/push is manual for now).
- A "prod" environment for the API — only "dev" exists today, matching
  `platform/db`'s existing single Terraform environment.

## Repo layout

```
platform/
  db/
    infrastructure/            # moved from tui/infrastructure — Azure Postgres Terraform
      modules/postgresql/
      environments/dev.cus.tfvars
servers/
  api/
    go.mod                     # module github.com/mrcunninghamz/money-bae/servers/api
    go.sum
    cmd/api/main.go            # entrypoint: load config, connect DB, AutoMigrate, start server
    internal/
      config/                  # env var loading
      database/                # gorm.Open(postgres...) wiring
      models/                  # Base + User
      auth/                    # OIDC JWT verification middleware (not wired in yet)
      httpapi/                 # router (net/http ServeMux) + health handler
    .env.local.example
    .env.dev.example
    use-local-env.sh
    use-dev-env.sh
    infrastructure/
      Dockerfile
      cdk/                     # TypeScript CDK app
        bin/api.ts
        lib/api-stack.ts
        cdk.json
        package.json
        tsconfig.json
```

`platform/db` and `servers/api` are new top-level siblings to `tui/`. The
`tui/infrastructure` move to `platform/db/infrastructure` is a plain
`git mv` — no content changes beyond what's described below. Nesting the
Terraform under an `infrastructure/` subfolder (rather than directly in
`platform/db/`) mirrors the convention `servers/api/infrastructure/` already
uses for that project's Dockerfile/CDK — every top-level project keeps its
IaC in its own `infrastructure/` subfolder.

## Go modules & workspace

Single Go module: `servers/api/go.mod`. `cmd/api` and all `internal/*`
packages live inside it — normal package layout, not a multi-module concern.

No `go.work` yet. A workspace earns its keep only when a second Go module in
this repo needs to import the first without a published version (e.g. a
future `servers/worker` sharing the data layer). Trigger for revisiting:
the day a second Go module appears anywhere under `servers/`.

The data layer (`internal/models`, `internal/database`) stays a **package**, not
its own module, for the same reason: there is exactly one consumer today
(the API binary itself). `internal/` also enforces this — Go's compiler
only lets code under `servers/api/` import it. It graduates to its own
module (and `go.work` ties things together) the day a second independent Go
binary needs it without needing the API's other dependencies.

## Data model (`internal/models/`)

```go
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

type User struct {
    Base
    Sub   string `gorm:"uniqueIndex;not null"` // OIDC subject claim — external identity key
    Email string `gorm:"index"`
}
```

- `ID` uses UUIDv7 (`github.com/google/uuid`'s `NewV7()`, available since
  v1.6.0), generated in Go via the `BeforeCreate` hook — GORM promotes hooks
  from embedded structs, so this fires for `User` and any future model
  embedding `Base`. No DB-side generation, so no Postgres extension needed;
  `uuid` is a native column type. `google/uuid.UUID` implements
  `sql.Scanner`/`driver.Valuer` natively, so it plugs into GORM with no
  wrapper package.
- UUID (not auto-increment int) because these IDs may be exposed in API
  responses/URLs; UUIDv7 keeps B-tree insert locality that random UUIDv4
  loses.
- `Sub` is unique + indexed — it's the lookup key on every authenticated
  request (find-or-create by `sub` from the verified token).
- `Email` isn't unique/not-null — OIDC's `email` claim is optional per spec;
  stored for display only.
- No soft-delete (`DeletedAt`) — nothing in scope needs it; add to `Base`
  later if a real need shows up.

## Auth middleware (`internal/auth/`) — built, not wired in

Uses `github.com/coreos/go-oidc/v3/oidc`, which handles OIDC discovery
(`GET {issuer}/.well-known/openid-configuration`) and JWKS fetching/caching/
rotation. Works unchanged against Auth0 or Microsoft Entra External ID,
since both are standard OIDC issuers — no vendor SDK.

```go
provider, err := oidc.NewProvider(ctx, cfg.OIDCIssuerURL)
verifier := provider.Verifier(&oidc.Config{ClientID: cfg.OIDCAudience})
```

What's actually validated is the caller's **access token** (a JWT, when the
API is registered as an audience/resource with the IdP), not an ID token —
ID tokens are for the client app, not the resource server. `ClientID` in
`oidc.Config` is really "expected audience" here: the API's own identifier,
not the frontend client's ID.

```go
func RequireAuth(verifier *oidc.IDTokenVerifier, users *UserStore) func(http.Handler) http.Handler
```

Extracts `Authorization: Bearer <token>`, verifies it, pulls `sub` (and
`email` if present) from the claims, then finds-or-creates the `User` by
`sub` and attaches `*models.User` to the request context. Verification and
provisioning are **not** split into separate steps — one middleware does
both, since there's no other service layer in this scaffold to own
provisioning separately, and splitting it would add a layer with nothing to
live in yet.

**Not wired into the router in this pass.** `oidc.NewProvider` does a live
network discovery call at startup — it would fail without a real issuer.
`cmd/api/main.go`/the router setup leaves the `RequireAuth(...)` wiring
commented out, with a note that it needs a real `OIDC_ISSUER_URL` and
`OIDC_AUDIENCE` before it can be enabled.

## HTTP layer & wiring (`internal/httpapi/`, `cmd/api/main.go`)

Router: stdlib `net/http`, using Go 1.22+'s `ServeMux` method+path pattern
matching (`"GET /health"`) — no third-party router needed for this scope.

```go
func NewRouter(db *gorm.DB) http.Handler {
    mux := http.NewServeMux()
    mux.HandleFunc("GET /health", healthHandler(db))
    // Once an IdP is configured:
    // mux.Handle("GET /me", auth.RequireAuth(verifier, userStore)(meHandler()))
    return mux
}
```

`/health` pings the DB (`sqlDB, _ := db.DB(); sqlDB.PingContext(ctx)`) —
returns 503 on failure, 200 `{"status":"ok"}` otherwise. This is deliberate:
it's what App Runner's health check polls, so it should catch a broken DB
connection, not just "process is alive."

```go
// cmd/api/main.go
cfg := config.Load()                    // DATABASE_URL, PORT, OIDC_ISSUER_URL, OIDC_AUDIENCE
db := database.Connect(cfg.DatabaseURL) // gorm.Open(postgres.Open(dsn), &gorm.Config{})
db.AutoMigrate(&models.User{})
router := httpapi.NewRouter(db)
http.ListenAndServe(":"+cfg.Port, router)
```

`internal/config` loads from process env. `PORT` defaults to `8080` — this
value must match the CDK stack's `imageConfiguration.port` (see
Infrastructure below).

### Local env files

`godotenv` (Go analog to `tui`'s `dotenvy`) has no C#-`appsettings`-style
environment layering built in — it's just `Load(files...)` with first-file-
wins precedence, and it never overrides a variable already set in the real
process environment ([joho/godotenv](https://github.com/joho/godotenv)).

Unlike `tui` (a binary that always runs on your own machine), the API gets
**deployed**. A real "dev" environment's config comes from the CDK stack's
App Runner environment/secrets config, not a checked-in file — so there's no
deployed-environment analog to a `.env.dev` file. What *is* useful locally
is being able to switch which resources your own machine points at (fully
local, the shared dev DB, or a mix) — so this mirrors `tui`'s copy-to-`.env`
convention directly, just renamed:

- Tracked templates: `.env.local.example`, `.env.dev.example` (each just a
  `DATABASE_URL=...` placeholder)
- Gitignored actual files: `.env.local`, `.env.dev`, `.env` (the one the app
  loads)
- Switch scripts (mirroring `tui`'s `use-dev-env.sh`/`use-prod-env.sh`):
  `use-local-env.sh` (`cp .env.local .env`), `use-dev-env.sh`
  (`cp .env.dev .env`)
- App always does a single `godotenv.Load()` — no multi-file layering
- A "mixture of local and dev resources" case needs no special handling:
  each `.env.<profile>` file is plain `KEY=value` content you edit — nothing
  stops `.env.local` from pointing `DATABASE_URL` at the shared dev DB while
  the binary runs on your machine. If a genuinely distinct third profile
  becomes a recurring need, add another `.env.<name>.example` + switch
  script the same way.

**Action needed**: add `.env.local` to the repo's root `.gitignore` (it
currently only lists `.env`, `.env.dev`, `.env.prod` from the `tui` days).

## Database

Reuses the existing Azure Postgres flexible server `psql-mb-core-cus-dev`
(in `rg-mb-core-cus-dev`) rather than provisioning a second server — no
added Azure cost for one more database on the same server. New database:
**`money_bae_api`**, alongside the existing `money_bae`. Schema is owned by
GORM (`AutoMigrate`), independent from `money_bae`'s Diesel-managed schema —
genuinely separate data, just co-located on the same server.

### `platform/db` Terraform changes

The `modules/postgresql` module currently creates exactly one
`azurerm_postgresql_flexible_server_database` resource tied to a single
`database_name` string variable. This changes to support multiple databases
per server:

- `variables.tf`: `database_name` (string) → `database_names`
  (`list(string)`, no default — explicit list).
- `main.tf`:
  ```hcl
  resource "azurerm_postgresql_flexible_server_database" "main" {
    for_each  = toset(var.database_names)
    name      = each.value
    server_id = azurerm_postgresql_flexible_server.main.id
    charset   = "UTF8"
    collation = "en_US.utf8"
  }
  ```
- Root `main.tf`: `database_names = ["money_bae", "money_bae_api"]`.
- `outputs.tf` (module and root): `database_name` (singular) →
  `database_names` (list) + `connection_strings` (map of name→URI). While
  rewriting this, fix a pre-existing bug in the current `connection_string`
  output — it has a stray extra `server.name` segment and uses `@` instead
  of `:` between login and password, so it isn't a valid Postgres URI as
  written:
  ```hcl
  output "connection_strings" {
    value = {
      for name, db in azurerm_postgresql_flexible_server_database.main :
      name => "postgresql://${var.administrator_login}:${var.administrator_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/${db.name}?sslmode=require"
    }
    sensitive = true
  }
  ```

**State safety (this is live infrastructure — confirmed, not assumed)**:
switching the database resource from a plain address to `for_each` changes
its state address (`...main` → `...main["money_bae"]`). Without help,
Terraform would plan to destroy and recreate the live `money_bae` database.
A `moved` block avoids that:
```hcl
moved {
  from = azurerm_postgresql_flexible_server_database.main
  to   = azurerm_postgresql_flexible_server_database.main["money_bae"]
}
```
This requires Terraform ≥1.1 (installed CLI here is 1.11.1); add
`required_version = ">= 1.1"` to `providers.tf` since it isn't currently
pinned. `terraform plan` must be reviewed to confirm it shows a *move*, not
a destroy/create, before `apply`.

**State backend — verified live, not migrated**: the backend documented in
`infrastructure/README.md` (storage account `stmbtfstateshared`, resource
group `rg-moneybae-tfstate-shared`, container `tfstate`, key
`core/dev.cus.tfstate`) already exists and is already in active use — this
was confirmed directly:
```
$ terraform init -backend-config=... -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
$ terraform state list
azurerm_resource_group.main
module.postgresql.azurerm_postgresql_flexible_server.main
module.postgresql.azurerm_postgresql_flexible_server_database.main
module.postgresql.azurerm_postgresql_flexible_server_firewall_rule.allow_all[0]
module.postgresql.azurerm_postgresql_flexible_server_firewall_rule.allow_azure_services
```
No state migration is needed. The README's documented `terraform init`
command was missing `-backend-config="subscription_id=..."` (required — the
first init attempt failed without it); this has already been fixed in
`tui/infrastructure/README.md` (to move to `platform/db/infrastructure/README.md`).

Note for future awareness: Terraform state stores resource values —
including `db_admin_password` — in plaintext regardless of a variable's
`sensitive = true` (that flag only masks CLI/plan output). Current Azure
RBAC on the storage account already appears scoped (blob listing was
denied under the working session's identity) — worth keeping it that way.

## Infrastructure (`servers/api/infrastructure/`)

### Dockerfile

Multi-stage; build context is `servers/api/` (`docker build -f
infrastructure/Dockerfile .` run from `servers/api/`). Simpler than `tui`'s
Rust build in one respect: `gorm.io/driver/postgres` is pure Go (no
`libpq`/CGO dependency the way Diesel's Rust driver needs), so this is a
fully static binary with no C toolchain in the runtime image:

```dockerfile
FROM golang:1.22-alpine AS builder
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /app/server ./cmd/api

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/server /server
ENTRYPOINT ["/server"]
```

Same Dockerfile is used for both local sanity-checks (`docker build` +
`docker run`) and the production artifact pushed to ECR — not two separate
files. Day-to-day local dev still just uses `go run`.

### AWS deployment target

App Runner, not Lambda, and not a full container orchestrator. Rough
Azure-familiar mapping:

| Azure | AWS equivalent | Ops overhead |
|---|---|---|
| App Service (Web App for Containers) | **App Runner** (chosen) | Lowest — point at an image, get HTTPS + a URL |
| Container Apps | ECS on Fargate | Medium — define task + service, wire up ALB yourself |
| AKS | EKS | Highest — full Kubernetes, not needed here |
| Azure Database for PostgreSQL | *(not used — API reuses the existing Azure DB instead of RDS)* | — |

Postgres deliberately stays on Azure rather than moving to RDS — the API
container (stateless, on AWS) connects to Azure Postgres over the network
via `DATABASE_URL`, the same pattern `tui` already uses today.

### CDK app — TypeScript, `@aws-cdk/aws-apprunner-alpha`

IaC split deliberately by cloud: Terraform for the Azure DB side
(`platform/db`), CDK for the AWS App Runner side
(`servers/api/infrastructure/cdk`).

App Runner has no stabilized CDK L2 construct yet — only the experimental
`@aws-cdk/aws-apprunner-alpha` package (versioned in lockstep with
`aws-cdk-lib`, can have breaking changes) versus the plain L1 `CfnService`
in core `aws-cdk-lib` (stable, but hand-written CloudFormation-shaped
properties, e.g. manual IAM policy JSON). Chose the alpha package — its
higher-level API is worth the version-lockstep maintenance.

```ts
// lib/api-stack.ts
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

- `ecr.Repository(...)` creates a **new** ECR repository (a named,
  versioned collection of container images — the AWS/CDK "Repository" here
  is unrelated to the data-access repository pattern used in
  `tui/src/repositories/`, or to a git repo; same word, different concept).
- `Secret.fromSecretNameV2(...)` **imports a reference** to a secret that
  must already exist (created out-of-band) — it does not create one.
- `Source.fromEcr(...)` also auto-creates the IAM role App Runner needs to
  pull from that specific private repo — hand-written with the L1 construct.
- `environmentSecrets` injects `DATABASE_URL` as a decrypted runtime env var
  fetched from Secrets Manager at container startup, and auto-grants the
  instance role `secretsmanager:GetSecretValue` scoped to just that secret.
  The Go app just reads `os.Getenv("DATABASE_URL")` — it never talks to
  Secrets Manager directly.
- `healthCheck` polls `GET /health` — the same DB-pinging endpoint from the
  HTTP layer section — to decide instance health/routing.
- A separate `bin/api.ts` entry point actually instantiates `ApiStack` under
  a `cdk.App()` — the class alone is inert without it.

**Manual bootstrap steps, outside CDK** (so the real secret value and image
never touch source control or the CFN template):
1. Create the secret's value: `aws secretsmanager create-secret --name
   money-bae-api/database-url --secret-string "postgresql://..."`.
2. Push the first image: `docker build` + `docker push` to the ECR repo
   `cdk deploy` creates — the service has nothing to run until an image
   exists at the `latest` tag.

## Deferred / out of scope (recap)

- Full CRUD for domain entities beyond `users`.
- IdP selection (Auth0 vs. Entra External ID) and enabling `RequireAuth`.
- CI/CD for image build/push.
- A "prod" environment/Terraform environment/App Runner stage for the API.
- `go.work` — until a second Go module exists in this repo.