# platform/db Subscription Migration — Design

**Issue:** [#60](https://github.com/mrcunninghamz/money-bae/issues/60)

## Context

`platform/db` provisions money-bae's shared Azure PostgreSQL Flexible
Server (`psql-mb-core-cus-dev`, resource group `rg-mb-core-cus-dev`),
holding two databases:

- `money_bae` — the Rust TUI application's data
- `money_bae_api` — the Go API's data

Both the server and its Terraform state backend
(`rg-moneybae-tfstate-shared`/`stmbtfstateshared`) live in the old Azure
subscription (`c6f1212c-ec19-425f-96a0-41f2db717ea8`). Identity work
(`platform/entra-external-id`, issue #57) already moved to a personal
subscription (`085f952f-488d-4c4d-bd33-0fcf8fd37e17`, tenant
`kmerecidolive.onmicrosoft.com`) where full admin rights are available,
using company-level (`kkbae`) naming. For money-bae's Azure footprint to
live under one subscription, and since this Postgres server is expected
to become shared dev infrastructure across the whole `kkb` organization
(not money-bae-exclusive), `platform/db` moves there too, adopting the
same `kkb` naming convention.

Unlike the identity work (no data yet), this server holds real user data,
so this is a genuine data migration, not a Terraform-only move.

## Goals

- New Postgres server + databases exist in the new subscription, with the
  same data as the old server today.
- Terraform state for `platform/db` lives in the existing company-level
  backend set up for issue #57, alongside the identity state.
- `platform/db`'s Terraform code drops its `infrastructure/` wrapper
  subfolder, matching `platform/api/`, `platform/web-client/`, and
  `platform/entra-external-id/` (only `platform/db/` used the wrapper).
- Every place that references the old server's hostname or the old
  subscription ID is updated.
- The old server and old state backend are left running, untouched, as a
  rollback safety net — decommissioning them is explicitly out of scope
  here (separate future follow-up).

## Non-goals

- Zero-downtime migration. This is a personal app with no live production
  traffic (App Runner has never actually been deployed — `cdk deploy` was
  only ever `cdk synth`'d), so a maintenance window is unnecessary
  ceremony; a straightforward dump/restore is fine.
- Destroying/decommissioning the old server or old state backend.
- Any change to `servers/api/cmd/import-legacy-data`'s logic — it's reused
  as-is, pointed at new source/target DBs.
- Automating the actual Azure resource creation, data copy, or secret
  rotation. Per `platform/db/CLAUDE.md`'s "never apply/destroy without
  human confirmation" rule (mirrored for `platform/entra-external-id`),
  this design produces committed code/doc changes plus a human-executed
  runbook (via the writing-plans skill, following #57's precedent) — no
  agent runs live `az`/`terraform apply`/`pg_dump`/`pg_restore` commands
  against real Azure resources.

## Target naming & backend

| | Old (subscription `c6f1212c-...`) | New (subscription `085f952f-...`) |
|---|---|---|
| Resource group | `rg-mb-core-cus-dev` | `rg-kkb-core-cus-dev` |
| Server | `psql-mb-core-cus-dev` | `psql-kkb-core-cus-dev` |
| Databases | `money_bae`, `money_bae_api` | `money_bae`, `money_bae_api` (unchanged — app-specific, not org-level) |
| DB admin login | `mbae` | `mbae` (unchanged — just a technical username) |
| State backend | storage `stmbtfstateshared`, RG `rg-moneybae-tfstate-shared`, key `core/dev.cus.tfstate` | storage `stkkbtfstatecus`, RG `rg-kkbae-tfstate-shared` (both already exist, provisioned for #57), key `db/dev.cus.tfstate` |

Only `app_short_name` changes (`mb` → `kkb`) in
`environments/dev.cus.tfvars`; `component`/`location_abrv`/`environment`
stay `core`/`cus`/`dev`. `platform/db/providers.tf`'s `subscription_id`
moves to the new subscription — from that point, this Terraform code
exclusively manages the new subscription's resources; the old server
becomes intentionally unmanaged by Terraform (its state blob is left
alone, untouched, for the future decommission follow-up).

The new backend key is a fresh, empty key — Terraform initializes there
with `-reconfigure` (not `-migrate-state`). There is nothing to migrate:
the new server is a distinct Azure resource in a different tenant, so
`terraform apply` against the empty new-key state creates it fresh.

## Directory restructuring

`platform/db/infrastructure/*` flattens up one level to `platform/db/*`:

```
platform/db/
├── CLAUDE.md
├── AGENTS.md
├── README.md            (was infrastructure/README.md)
├── main.tf               (was infrastructure/main.tf)
├── providers.tf          (was infrastructure/providers.tf)
├── variables.tf          (was infrastructure/variables.tf)
├── outputs.tf            (was infrastructure/outputs.tf)
├── environments/
│   └── dev.cus.tfvars
└── modules/
    └── postgresql/
```

`platform/db/CLAUDE.md`/`AGENTS.md` drop the "Terraform lives in
`infrastructure/`, run commands from there" notes — Terraform commands
now run directly from `platform/db/`.

## Data migration approach

1. **`money_bae`** (tui data, source of truth): `pg_dump -Fc` off the old
   server (reusing `tui/backup-db.sh prod`, which already does exactly
   this), then `pg_restore` into the new server's `money_bae`. A
   byte-for-byte copy.
2. **`money_bae_api`**: re-run `servers/api/cmd/import-legacy-data` with
   `SOURCE_DATABASE_URL` = new server's (just-restored) `money_bae` and
   `DATABASE_URL` = new server's fresh, empty `money_bae_api`. This
   reproduces the API's derived data rather than copying the old
   `money_bae_api` byte-for-byte.

   This is safe/equivalent to a byte-for-byte copy specifically *because*
   App Runner has never been deployed live — `money_bae_api` on the old
   server has no data beyond what the original `import-legacy-data` run
   produced, so regenerating it via the same importer loses nothing. If
   App Runner is ever actually deployed against the old server before
   this migration happens, this assumption must be re-checked (a
   byte-for-byte `money_bae_api` dump/restore would be the safer
   fallback in that case).

## In-repo changes (committed as part of this work, no live effect until the runbook is run)

- Directory flatten described above.
- `platform/db/providers.tf` — new `subscription_id`.
- `platform/db/environments/dev.cus.tfvars` — `app_short_name = "kkb"`.
- `platform/db/README.md` — new backend-config command (new storage
  account/RG/key/subscription), new resource names.
- `platform/db/CLAUDE.md`/`AGENTS.md` — new subscription ID, new resource
  names, flattened directory note, and a note that this is now
  kkb-org-level shared dev infrastructure (not money-bae-exclusive),
  mirroring `platform/entra-external-id/CLAUDE.md`'s framing of the CIAM
  tenant as company-level shared infra.
- `servers/api/.env.dev.example` — update the literal
  `psql-mb-core-cus-dev.postgres.database.azure.com` example hostname to
  the new one.
- `tui/CLAUDE.md`, `tui/AGENTS.md`, `tui/README.md` — same hostname
  update in their example connection strings.

## Manual cutover steps (human-run; not repo files)

- **tui**: edit `~/.config/money-bae/money-bae.toml`'s
  `database_connection_string`, and `tui/.env.prod`'s `DATABASE_URL`, to
  the new server's connection string.
- **servers/api**: edit local `.env.dev`'s `DATABASE_URL` to the new
  server's connection string.
- **platform/api** (App Runner): check whether the Secrets Manager secret
  `money-bae-api/database-url` exists yet (it may not, since `cdk deploy`
  has never been run); update it if it does.

## Out of scope / follow-up

Decommissioning the old server (`psql-mb-core-cus-dev`) and old state
backend (`rg-moneybae-tfstate-shared`/`stmbtfstateshared`) is a separate,
later piece of work, done once the new server has proven solid. It is not
part of this design.

## Risks

- **Fresh-server reimport, not byte-for-byte, for `money_bae_api`**:
  covered above — safe today because App Runner has never run live
  against real traffic. Called out explicitly so it's re-verified if that
  changes before this migration executes.
- **No automated rollback**: since the old server/backend are left
  intact and untouched, rollback is simply "keep pointing everything at
  the old server" — no destructive step in this design forecloses that
  option.
- **Manual, per-machine cutover for tui**: there's no CI/CD for tui, so
  the connection-string update is a manual step on whatever machine(s)
  run it in production. Easy to miss; the plan should call this out as an
  explicit, unmissable step.
