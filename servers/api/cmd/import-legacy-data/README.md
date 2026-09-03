# import-legacy-data

One-time ETL: moves the old TUI's data (`money_bae` database, Diesel schema,
integer primary keys, single-user) into the new API's schema (`money_bae_api`,
GORM models, UUIDv7 primary keys, multi-tenant). See issue #47 for the full
design.

Every row gets associated with one seeded `User` (static ID, see
`main.go`). Integer legacy IDs are remapped to new UUIDs, with all foreign
keys (`ledger_bills.bill_id`/`ledger_id`, `incomes.ledger_id`,
`pto_plan.pto_id`, `holiday_hours.pto_id`) rewritten to match.

**Read-only against the source.** This tool only ever issues `SELECT`
queries against `SOURCE_DATABASE_URL` — it never writes to the legacy
database. All writes go to the target (`DATABASE_URL`).

## Before running

Take a backup of the legacy database as a safety net, even though this tool
never writes to it — cheap insurance before running anything new near real
data:

```bash
cd ../../../tui
./backup-db.sh prod
```

## Running

Two connection strings are required:

- `DATABASE_URL` — the target `money_bae_api` database. Uses the same
  `.env.local`/`.env.dev` + `use-local-env.sh`/`use-dev-env.sh` convention
  as `cmd/api` (see `../../README.md`).
- `SOURCE_DATABASE_URL` — the legacy `money_bae` database, read-only. Not
  part of the tracked `.env.*.example` templates (this is a one-time tool,
  not a normal part of running the API) — set it directly:

```bash
./use-local-env.sh   # or use-dev-env.sh, sets DATABASE_URL (the target)
export SOURCE_DATABASE_URL="postgres://<user>:<password>@<host>/money_bae?sslmode=require"
go run ./cmd/import-legacy-data
```

The target schema is migrated automatically (via `internal/migrations`) if
it doesn't already exist.