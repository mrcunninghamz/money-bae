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
