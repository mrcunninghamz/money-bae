# Money-Bae - Terraform Infrastructure

Terraform configuration for deploying Money-Bae core infrastructure on Azure.

## Architecture

- **Resource Group**: `rg-mb-core-cus-dev`
- **PostgreSQL Flexible Server**: `psql-mb-core-cus-dev`, holding two databases:
  - `money_bae` — the Rust TUI application's data
  - `money_bae_api` — the Go API's data (independent schema, same server)

## ⚠️ Live Infrastructure

This Terraform manages real Azure resources with real remote state. Never
run `terraform apply` or `terraform destroy` without first reviewing the
`terraform plan` output carefully — this is not a sandbox.

`zone = "1"` on the PostgreSQL server is pinned to match the live server's
actual current availability zone (Azure auto-assigned it at creation, before
this config tracked it) — without this, `terraform plan` would show an
unrelated in-place update clearing it on every apply.

## Prerequisites

- Azure CLI authenticated (`az login`)
- Terraform >= 1.1 (required for the `moved` block used in `modules/postgresql/main.tf`)
- Azure subscription with permissions to create resources
- Remote state storage account (already provisioned: `stmbtfstateshared`)
- `TF_VAR_money_bae_db_admin_password` set in your environment before running `plan`/`apply` — see below. Never pass it as a `-var` flag or write it into any file (including `.tfvars`); the admin login (`money_bae_db_admin_login`) is set in `environments/dev.cus.tfvars` — it's still marked `sensitive` in Terraform (so it stays out of plan/apply console output), even though it's a non-secret username value that's fine to commit.

### Setting the admin password

Add it to your shell profile (e.g. `~/.zshenv`) so it's set for every new shell, rather than exporting it ad hoc:

```bash
echo 'export TF_VAR_money_bae_db_admin_password="<the real password>"' >> ~/.zshenv
```

Terraform picks up `TF_VAR_<name>` environment variables automatically for any declared variable named `<name>` — no `-var` flag needed.

## Local Deployment

All commands below assume you're in this directory (`platform/db/infrastructure/`).

### 1. Initialize Backend

Configure remote state storage:

```bash
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=core/dev.cus.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
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
infrastructure/
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
