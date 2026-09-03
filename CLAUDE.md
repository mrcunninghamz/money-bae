# CLAUDE.md

money-bae is a personal finance app, structured as a monorepo:

- `tui/` — the original Rust terminal UI application. See `tui/CLAUDE.md`.
- `servers/api/` — a Go + GORM API for a future web/mobile frontend. See `servers/api/CLAUDE.md`.
- `servers/local-dev/` — a docker-compose Postgres for local API development. See `servers/local-dev/README.md`.
- `platform/db/` — Terraform for the shared Azure Postgres server backing both `tui/` and `servers/api/`. See `platform/db/CLAUDE.md`.

Each project's own CLAUDE.md/AGENTS.md/README.md has the details for working within it.
