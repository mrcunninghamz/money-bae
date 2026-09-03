# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this directory.

## Project Overview

`platform/db` is the Terraform (IaC) for money-bae's shared Azure PostgreSQL Flexible Server. It provisions a single server (`psql-mb-core-cus-dev`) holding two independent databases:

- `money_bae` — the Rust TUI application's data (see `../../tui/`)
- `money_bae_api` — the Go API's data (see `../../servers/api/`)

The Terraform itself lives in `infrastructure/`, mirroring the convention `servers/api/infrastructure/` uses for that project's own IaC (Dockerfile/CDK) — every top-level project keeps its IaC in its own `infrastructure/` subfolder.

## ⚠️ This Is Live Infrastructure

Real Azure resources, real remote Terraform state (Azure storage account `stmbtfstateshared`, resource group `rg-moneybae-tfstate-shared`, container `tfstate`). Never run `terraform apply` or `terraform destroy` without explicit human confirmation of the `terraform plan` output first. `terraform plan`/`validate`/`fmt`/`state list` are safe to run freely.

## Setup

Full instructions: `infrastructure/README.md`. Key points:

- Requires `TF_VAR_money_bae_db_admin_password` set in your environment (e.g. added to `~/.zshenv`) before running `plan`/`apply` — never pass it as a `-var` flag or write it into any file.
- Requires Terraform >= 1.1 (the module uses a `moved` block).
- Backend init requires `-backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"` — the README's documented command already includes this.

## Working Directory

All `terraform` commands run from `infrastructure/`, not from `platform/db/` itself.
