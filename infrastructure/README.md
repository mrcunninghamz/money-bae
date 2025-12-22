# Money-Bae - Terraform Infrastructure

Terraform configuration for deploying Money-Bae core infrastructure on Azure.

## Architecture

- **Resource Group**: `rg-mb-core-cus-dev`
- **PostgreSQL Database**: (future) Flexible Server for TUI application data

## Prerequisites

- Azure CLI authenticated (`az login`)
- Terraform >= 1.0
- Azure subscription with permissions to create resources
- Remote state storage account (created separately)

## Local Deployment

### 1. Initialize Backend

Configure remote state storage:

```bash
cd infrastructure

terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=core/dev.cus.tfstate"
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

## Module Structure

```
infrastructure/
├── main.tf              # Main orchestration
├── providers.tf         # Azure provider config
├── variables.tf         # Input variables
├── outputs.tf           # Output values
├── environments/        # Environment-specific tfvars
│   └── dev.cus.tfvars
└── modules/             # Reusable modules (future)
    └── postgresql/      # PostgreSQL Flexible Server module
```

## Destroy

```bash
terraform destroy -var-file="environments/dev.cus.tfvars"
```
