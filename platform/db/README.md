# platform/db — Terraform Infrastructure

Terraform configuration for a shared Azure PostgreSQL Flexible Server. This
is `kkb`-org-level shared dev infrastructure, not exclusive to money-bae —
the server may host other `kkb`-org databases in the future, the way
`platform/entra-external-id`'s CIAM tenant is shared company-level identity
infrastructure rather than a money-bae-only resource.

## Architecture

- **Resource Group**: `rg-kkb-core-cus-dev`
- **PostgreSQL Flexible Server**: `psql-kkb-core-cus-dev`, holding two databases:
  - `money_bae` — the Rust TUI application's data
  - `money_bae_api` — the Go API's data (independent schema, same server)

## ⚠️ Live Infrastructure

This Terraform manages real Azure resources with real remote state. Never
run `terraform apply` or `terraform destroy` without first reviewing the
`terraform plan` output carefully — this is not a sandbox.

`zone = "1"` in `modules/postgresql/main.tf` is now a deliberate choice for
the fresh server this migration creates (not a reconciliation of an
existing resource, as it was for the old server) — availability-zone
offerings are per-subscription-per-region, so if `terraform apply` fails
because zone 1 isn't offered for this SKU in the new subscription, try a
different zone value or remove the `zone` argument entirely to let Azure
auto-assign one.

## Prerequisites

- Azure CLI authenticated (`az login`)
- Terraform >= 1.1 (required for the `moved` block used in `modules/postgresql/main.tf`)
- Azure subscription with permissions to create resources
- Remote state storage account (already provisioned: `stkkbtfstatecus`,
  the same kkb-org company-level backend used by `platform/entra-external-id`)
- That state backend is provisioned by `platform/entra-external-id`'s own
  setup (issue #57) — that setup must already exist before this project's
  `terraform init` can succeed
- `TF_VAR_money_bae_db_admin_password` set in your environment before running `plan`/`apply` — see below. Never pass it as a `-var` flag or write it into any file (including `.tfvars`); the admin login (`money_bae_db_admin_login`) is set in `environments/dev.cus.tfvars` — it's still marked `sensitive` in Terraform (so it stays out of plan/apply console output), even though it's a non-secret username value that's fine to commit.

### Setting the admin password

Add it to your shell profile (e.g. `~/.zshenv`) so it's set for every new shell, rather than exporting it ad hoc:

```bash
echo 'export TF_VAR_money_bae_db_admin_password="<the real password>"' >> ~/.zshenv
```

Terraform picks up `TF_VAR_<name>` environment variables automatically for any declared variable named `<name>` — no `-var` flag needed.

## Local Deployment

All commands below assume you're in this directory (`platform/db/`).

### 1. Initialize Backend

Configure remote state storage:

```bash
terraform init \
  -backend-config="resource_group_name=rg-kkbae-tfstate-shared" \
  -backend-config="storage_account_name=stkkbtfstatecus" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=db/dev.cus.tfstate" \
  -backend-config="subscription_id=085f952f-488d-4c4d-bd33-0fcf8fd37e17"
```

### 2. Plan

```bash
terraform plan -var-file="environments/dev.cus.tfvars"
```

### 3. Apply

```bash
terraform apply -var-file="environments/dev.cus.tfvars"
```

## CI/CD Pipeline

Infrastructure deployment will be handled by Azure DevOps pipeline (future).

## Outputs

- `resource_group_name`: Resource group name
- `resource_group_location`: Resource group location
- `postgresql_server_name`: Name of the PostgreSQL Flexible Server
- `postgresql_server_fqdn`: FQDN of the PostgreSQL Flexible Server
- `postgresql_database_names`: Names of the PostgreSQL databases on the server
- `postgresql_connection_strings`: Map of database name to PostgreSQL connection string (sensitive)

## Module Structure

```
platform/db/
├── main.tf              # Main orchestration
├── providers.tf         # Azure provider config
├── variables.tf         # Input variables
├── outputs.tf           # Output values
├── environments/        # Environment-specific tfvars
│   └── dev.cus.tfvars
└── modules/             # Reusable modules
    └── postgresql/      # PostgreSQL Flexible Server module
```

## Destroy

```bash
terraform destroy -var-file="environments/dev.cus.tfvars"
```
