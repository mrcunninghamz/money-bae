# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this directory.

## Project Overview

`platform/db` is the Terraform (IaC) for a shared Azure PostgreSQL Flexible Server. It provisions a single server (`psql-kkb-core-cus-dev`) holding two independent databases:

- `money_bae` — the Rust TUI application's data (see `../../tui/`)
- `money_bae_api` — the Go API's data (see `../../servers/api/`)

This is `kkb`-org-level shared dev infrastructure, not exclusive to money-bae — the server may host other `kkb`-org databases in the future, the way `platform/entra-external-id`'s CIAM tenant is shared company-level identity infrastructure rather than a money-bae-only resource. The Terraform lives directly in this directory (no `infrastructure/` wrapper subfolder), matching `platform/api/`, `platform/web-client/`, and `platform/entra-external-id/`.

## ⚠️ This Is Live Infrastructure

Real Azure resources, real remote Terraform state (Azure storage account `stkkbtfstatecus`, resource group `rg-kkbae-tfstate-shared`, container `tfstate`, key `db/dev.cus.tfstate` — the same company-level backend `platform/entra-external-id` uses for its own state, under a different key). Never run `terraform apply` or `terraform destroy` without explicit human confirmation of the `terraform plan` output first. `terraform plan`/`validate`/`fmt`/`state list` are safe to run freely.

## Setup

Full instructions: `README.md`. Key points:

- Requires `TF_VAR_money_bae_db_admin_password` set in your environment (e.g. added to `~/.zshenv`) before running `plan`/`apply` — never pass it as a `-var` flag or write it into any file.
- Requires Terraform >= 1.1 (the module uses a `moved` block).
- Backend init requires `-backend-config="subscription_id=085f952f-488d-4c4d-bd33-0fcf8fd37e17"` — the README's documented command already includes this.

## Working Directory

All `terraform` commands run from `platform/db/` (this directory) directly.
