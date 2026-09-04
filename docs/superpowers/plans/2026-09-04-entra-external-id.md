# Entra External ID Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provision a Microsoft Entra External ID (CIAM) tenant and four app registrations (`money-bae-local`, `money-bae-dev`, `money-bae-api-local`, `money-bae-api-dev`) as Terraform, with a working SPA→API scope grant, then point the Postman collection at OAuth2 PKCE using the new SPA client IDs.

**Architecture:** Two independent Terraform root modules under `platform/entra-external-id/` — `tenant/` (creates the CIAM tenant via the `azapi` provider, subscription-scoped auth) and `app-registrations/` (creates the four app registrations via the `azuread` provider, tenant-scoped auth, using two separate reusable modules — `api-registration` and `spa-registration` — instantiated once per environment each and wired together). Terraform can't compute one provider's config from a resource created in the same apply, so these must be separate states applied in order with a manual re-login step between them. This plan writes and validates the Terraform (`fmt`, `validate`) as its test cycle — there's no unit-test framework for HCL, and `terraform apply` against real Azure/Entra is a human-run, plan-reviewed step per this repo's live-infrastructure convention, never something a task runs unsupervised.

**Tech Stack:** Terraform >= 1.1, `hashicorp/azurerm` (existing pin), `azure/azapi`, `hashicorp/azuread`, `hashicorp/random`.

**Spec:** `docs/superpowers/specs/2026-09-04-entra-external-id-design.md`

## Global Constraints

- Terraform `>= 1.1` (matches `platform/db`).
- Provider pins (exact, matching `platform/db`'s pinning style): `azurerm = "=4.1.0"`, `azapi = "=2.12.0"` (tenant/ only), `azuread = "=3.9.0"` and `random = "=3.9.0"` (app-registrations/ only).
- Azure subscription for the tenant/ resource group: `c6f1212c-ec19-425f-96a0-41f2db717ea8` (hardcoded in `provider "azurerm"`, same as `platform/db/infrastructure/providers.tf`).
- Shared remote state backend: storage account `stmbtfstateshared`, resource group `rg-moneybae-tfstate-shared`, container `tfstate` — new keys `identity/tenant.tfstate` and `identity/app-registrations.tfstate`.
- Both root configs (`tenant/`, `app-registrations/`) initialize against this **real** backend — `terraform init` with the actual `-backend-config` flags (matching `platform/db`'s README exactly, including `-backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"`), not `-backend=false`. The `subscription_id` in the backend config targets the right subscription regardless of which subscription `az account show` currently reports as default — this is how `platform/db` already handles it. `terraform init` (even against a real backend) only creates/reads Terraform's own state-tracking metadata; it never creates, modifies, or destroys an Azure/Entra resource, so it's as safe to automate as `validate`/`plan`/`fmt`. Only `apply`/`destroy` need the human-review gate below. Modules (`api-registration`, `spa-registration`, and anything under `tenant/modules/` if added later) never get a backend block — Terraform hard-errors on that for non-root modules — so their standalone validation still uses `-backend=false`, which is unrelated to this real-backend requirement.
- No `infrastructure/` wrapper subfolder — `platform/entra-external-id/tenant/` and `platform/entra-external-id/app-registrations/` sit directly under `platform/entra-external-id/`, matching `platform/api/` and `platform/web-client/` (only `platform/db/` uses the wrapper).
- Region: `centralus` / `cus` for the resource group, matching `platform/db`.
- **Never run `terraform apply` or `terraform destroy` without a human reviewing the `terraform plan` output first** — this is real, live Azure/Entra infrastructure. No task in this plan runs `apply`; every task stops at `terraform validate`/`plan`-safe commands, and the real `init`/`plan`/`apply` sequence is documented as a manual runbook for the human to execute themselves (Task 4).
- App registration display names are exactly `money-bae-local`, `money-bae-dev`, `money-bae-api-local`, `money-bae-api-dev` (as specified by the user) — not derived from the `<type>-<app>-<component>-<loc>-<env>` Azure-resource naming convention, which doesn't apply to Graph objects.

---

## Task 1: `tenant/` — CIAM tenant Terraform

**Files:**
- Create: `platform/entra-external-id/tenant/providers.tf`
- Create: `platform/entra-external-id/tenant/variables.tf`
- Create: `platform/entra-external-id/tenant/main.tf`
- Create: `platform/entra-external-id/tenant/outputs.tf`
- Create: `platform/entra-external-id/tenant/environments/shared.tfvars`

**Interfaces:**
- Produces: Terraform output `tenant_id` (string, GUID) and `tenant_domain` (string, `<name>.onmicrosoft.com`) — consumed manually by the human when running `app-registrations/` (Task 3), passed as `-var="ciam_tenant_id=..."`.

- [ ] **Step 1: Write `providers.tf`**

```hcl
terraform {
  required_version = ">= 1.1"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=4.1.0"
    }
    azapi = {
      source  = "azure/azapi"
      version = "=2.12.0"
    }
  }

  backend "azurerm" {}
}

provider "azurerm" {
  subscription_id = "c6f1212c-ec19-425f-96a0-41f2db717ea8"
  features {}
  resource_provider_registrations = "none"
}

provider "azapi" {}
```

- [ ] **Step 2: Write `variables.tf`**

```hcl
variable "app_short_name" {
  type        = string
  description = "Company short name (this is company-level shared infrastructure, not product-level — hence 'kkb' for company kkbae, not 'mb' for the money-bae product)"
  default     = "kkb"
}

variable "location" {
  type        = string
  description = "Azure region for the resource group holding the CIAM tenant resource"
  default     = "centralus"
}

variable "location_abrv" {
  type        = string
  description = "Abbreviated location name (eus, cus, wus)"
  default     = "cus"
}

variable "ciam_data_location" {
  type        = string
  description = "CIAM tenant data residency location: one of 'United States', 'Europe', 'Asia Pacific', 'Australia'"
  default     = "United States"
}

variable "ciam_country_code" {
  type        = string
  description = "Country code for the CIAM tenant (e.g. US)"
  default     = "US"
}

variable "ciam_tenant_name" {
  type        = string
  description = "Globally-unique tenant name: alphanumeric only, max 26 chars. Becomes <name>.onmicrosoft.com and <name>.ciamlogin.com"
  default     = "kkbaeexternalid"
}

variable "ciam_display_name" {
  type        = string
  description = "Display name shown for the CIAM tenant"
  default     = "KKBAE"
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}
```

- [ ] **Step 3: Write `main.tf`**

```hcl
resource "azurerm_resource_group" "identity" {
  name     = "rg-${var.app_short_name}-identity-${var.location_abrv}-shared"
  location = var.location
  tags     = var.tags
}

resource "azapi_resource" "ciam_tenant" {
  type      = "Microsoft.AzureActiveDirectory/ciamDirectories@2023-05-17-preview"
  name      = var.ciam_tenant_name
  parent_id = azurerm_resource_group.identity.id
  location  = var.ciam_data_location
  tags      = var.tags

  body = {
    properties = {
      createTenantProperties = {
        countryCode = var.ciam_country_code
        displayName = var.ciam_display_name
      }
    }
    sku = {
      name = "Standard"
      tier = "A0"
    }
  }

  response_export_values = ["properties.tenantId"]
}
```

- [ ] **Step 4: Write `outputs.tf`**

```hcl
output "tenant_id" {
  value       = azapi_resource.ciam_tenant.output.properties.tenantId
  description = "Tenant ID (GUID) of the new Entra External ID (CIAM) tenant"
}

output "tenant_domain" {
  value       = "${var.ciam_tenant_name}.onmicrosoft.com"
  description = "Default domain of the new CIAM tenant"
}

output "resource_group_name" {
  value = azurerm_resource_group.identity.name
}
```

- [ ] **Step 5: Write `environments/shared.tfvars`**

```hcl
app_short_name     = "kkb"
location           = "centralus"
location_abrv      = "cus"
ciam_data_location = "United States"
ciam_country_code  = "US"
ciam_tenant_name   = "kkbaeexternalid"
ciam_display_name  = "KKBAE"

tags = {
  Environment = "shared"
  Application = "MoneyBae"
  Component   = "Identity"
  ManagedBy   = "Terraform"
}
```

- [ ] **Step 6: Format, initialize against the real backend, and validate**

Run from `platform/entra-external-id/tenant/`:

```bash
terraform fmt -check
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=identity/tenant.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
terraform validate
```

This matches `platform/db`'s own `terraform init` command exactly (same storage account/resource group/container, different `key`) — real backend, not `-backend=false`. It requires `az login` with an identity that has access to that storage account; the `subscription_id` backend-config value targets the right subscription regardless of whichever subscription `az account show` currently reports as the CLI's default. `terraform init` only sets up Terraform's own state-tracking metadata in that storage account (creating the `identity/tenant.tfstate` blob entry if it doesn't exist yet) — it does not create, modify, or read any Azure/Entra resource, so it carries the same safety profile as `validate`/`plan`/`fmt`.

Expected: `fmt -check` prints nothing (already formatted); `init` succeeds (downloads `azurerm`/`azapi` provider plugins and reports successfully configured backend); `validate` prints `Success! The configuration is valid.`

If `fmt -check` reports a diff, run `terraform fmt` (no `-check`) to fix it, then re-run `-check` to confirm. If `init` fails with an authorization error against the storage account, STOP and report BLOCKED — that's a real access problem for the controller to resolve, not something to work around.

- [ ] **Step 7: Commit**

```bash
git add platform/entra-external-id/tenant/
git commit -m "Add Terraform for the Entra External ID (CIAM) tenant"
```

---

## Task 2: `api-registration` and `spa-registration` — two separate reusable modules

Split into two modules, not one combined pair — matching the pattern used
elsewhere for this kind of infra (one `api-registration` module type,
reusable per environment or even per consumer; a separate
`spa-registration` module type that takes an API's `client_id`/`scope_id`
as inputs rather than creating its own). This also means a future SPA or
tool could be pointed at an existing API module instance without
duplicating the API's definition.

**Files:**
- Create: `platform/entra-external-id/app-registrations/modules/api-registration/variables.tf`
- Create: `platform/entra-external-id/app-registrations/modules/api-registration/main.tf`
- Create: `platform/entra-external-id/app-registrations/modules/api-registration/outputs.tf`
- Create: `platform/entra-external-id/app-registrations/modules/spa-registration/variables.tf`
- Create: `platform/entra-external-id/app-registrations/modules/spa-registration/main.tf`
- Create: `platform/entra-external-id/app-registrations/modules/spa-registration/outputs.tf`

**Interfaces:**
- `api-registration` consumes: `var.display_name` (string, e.g. `"money-bae-api-local"`). Produces: outputs `client_id`, `identifier_uri`, `scope_id` (the raw scope UUID), `service_principal_object_id` (all string) — consumed by `spa-registration` module instances in Task 3.
- `spa-registration` consumes: `var.display_name` (string), `var.redirect_uris` (list(string)), `var.api_client_id`, `var.api_scope_id`, `var.api_service_principal_object_id` (all string, from an `api-registration` instance). Produces: output `client_id` — consumed by Task 3's root outputs.

- [ ] **Step 1: Write `api-registration/variables.tf`**

```hcl
variable "display_name" {
  type        = string
  description = "Display name for the API app registration (e.g. money-bae-api-local)"
}
```

- [ ] **Step 2: Write `api-registration/main.tf`**

```hcl
resource "random_uuid" "access_as_user" {}

resource "azuread_application" "api" {
  display_name     = var.display_name
  sign_in_audience = "AzureADMyOrg"
  identifier_uris  = ["api://${var.display_name}"]

  api {
    requested_access_token_version = 2

    oauth2_permission_scope {
      id                         = random_uuid.access_as_user.result
      value                      = "access_as_user"
      type                       = "User"
      enabled                    = true
      admin_consent_description  = "Allow the app to access ${var.display_name} on behalf of the signed-in user"
      admin_consent_display_name = "Access ${var.display_name}"
      user_consent_description   = "Allow the app to access ${var.display_name} on your behalf"
      user_consent_display_name  = "Access ${var.display_name}"
    }
  }
}

resource "azuread_service_principal" "api" {
  client_id = azuread_application.api.client_id
}
```

- [ ] **Step 3: Write `api-registration/outputs.tf`**

```hcl
output "client_id" {
  value = azuread_application.api.client_id
}

output "identifier_uri" {
  value = one(azuread_application.api.identifier_uris)
}

output "scope_id" {
  value = random_uuid.access_as_user.result
}

output "service_principal_object_id" {
  value = azuread_service_principal.api.object_id
}
```

- [ ] **Step 4: Format and validate `api-registration` standalone**

Run from `platform/entra-external-id/app-registrations/modules/api-registration/`:

```bash
terraform fmt -check
terraform init -backend=false
terraform validate
```

Expected: `fmt -check` clean, `init` downloads `azuread`/`random` plugins, `validate` prints `Success! The configuration is valid.` (Terraform treats any directory as a standalone root for `validate` purposes, so this works even though the directory is meant to be consumed as a module.)

- [ ] **Step 5: Write `spa-registration/variables.tf`**

```hcl
variable "display_name" {
  type        = string
  description = "Display name for the SPA app registration (e.g. money-bae-local)"
}

variable "redirect_uris" {
  type        = list(string)
  description = "Redirect URIs for the SPA (PKCE) platform"
}

variable "api_client_id" {
  type        = string
  description = "Client ID of the API app registration this SPA is granted access to (an api-registration module instance's client_id output)"
}

variable "api_scope_id" {
  type        = string
  description = "Scope UUID this SPA requests (an api-registration module instance's scope_id output)"
}

variable "api_service_principal_object_id" {
  type        = string
  description = "Object ID of the API app registration's service principal, needed to grant the delegated permission (an api-registration module instance's service_principal_object_id output)"
}
```

- [ ] **Step 6: Write `spa-registration/main.tf`**

```hcl
resource "azuread_application" "spa" {
  display_name     = var.display_name
  sign_in_audience = "AzureADMyOrg"

  single_page_application {
    redirect_uris = var.redirect_uris
  }

  required_resource_access {
    resource_app_id = var.api_client_id

    resource_access {
      id   = var.api_scope_id
      type = "Scope"
    }
  }
}

resource "azuread_service_principal" "spa" {
  client_id = azuread_application.spa.client_id
}

resource "azuread_service_principal_delegated_permission_grant" "spa_to_api" {
  service_principal_object_id          = azuread_service_principal.spa.object_id
  resource_service_principal_object_id = var.api_service_principal_object_id
  claim_values                         = ["access_as_user"]
}
```

- [ ] **Step 7: Write `spa-registration/outputs.tf`**

```hcl
output "client_id" {
  value = azuread_application.spa.client_id
}
```

- [ ] **Step 8: Format and validate `spa-registration` standalone**

Run from `platform/entra-external-id/app-registrations/modules/spa-registration/`:

```bash
terraform fmt -check
terraform init -backend=false
terraform validate
```

Expected: same as Step 4.

- [ ] **Step 9: Commit**

```bash
git add platform/entra-external-id/app-registrations/modules/
git commit -m "Add api-registration and spa-registration Terraform modules"
```

---

## Task 3: `app-registrations/` root — instantiate both environments

**Files:**
- Create: `platform/entra-external-id/app-registrations/providers.tf`
- Create: `platform/entra-external-id/app-registrations/variables.tf`
- Create: `platform/entra-external-id/app-registrations/main.tf`
- Create: `platform/entra-external-id/app-registrations/outputs.tf`
- Create: `platform/entra-external-id/app-registrations/environments/shared.tfvars`

**Interfaces:**
- Consumes: `api-registration` and `spa-registration` modules from Task 2 (see their Interfaces above).
- Produces: root outputs `spa_client_id_local`, `spa_client_id_dev`, `api_client_id_local`, `api_client_id_dev`, `api_identifier_uri_local`, `api_identifier_uri_dev` — read via `terraform output` by the human (Task 4's runbook) and referenced by name in Task 5's Postman docs.

- [ ] **Step 1: Write `providers.tf`**

```hcl
terraform {
  required_version = ">= 1.1"

  required_providers {
    azuread = {
      source  = "hashicorp/azuread"
      version = "=3.9.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "=3.9.0"
    }
  }

  backend "azurerm" {}
}

provider "azuread" {
  tenant_id = var.ciam_tenant_id
}
```

- [ ] **Step 2: Write `variables.tf`**

```hcl
variable "ciam_tenant_id" {
  type        = string
  description = "Tenant ID of the Entra External ID (CIAM) tenant — pass with -var, sourced from `terraform -chdir=../tenant output -raw tenant_id`. Not stored in tfvars since it's an output of a different Terraform state, not a static config value."
}

variable "local_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the local environment"
  default     = "http://localhost:3000"
}

variable "dev_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the dev environment: the web client's deployed CloudFront domain. Update this after platform/web-client's first `cdk deploy` — the distribution has no custom domain, so its domain is only known post-deploy."
  default     = "https://REPLACE-AFTER-FIRST-WEB-CLIENT-DEPLOY.cloudfront.net"
}
```

- [ ] **Step 3: Write `main.tf`**

```hcl
locals {
  # oauth.pstmn.io is Postman's OAuth2 callback proxy, added to every SPA
  # app's redirect URIs so the Postman collection (Task 5) can obtain real
  # tokens for manual API testing without a second app registration.
  postman_callback_redirect_uri = "https://oauth.pstmn.io/v1/callback"
}

module "api_local" {
  source = "./modules/api-registration"

  display_name = "money-bae-api-local"
}

module "api_dev" {
  source = "./modules/api-registration"

  display_name = "money-bae-api-dev"
}

module "spa_local" {
  source = "./modules/spa-registration"

  display_name                    = "money-bae-local"
  redirect_uris                   = [var.local_redirect_uri, local.postman_callback_redirect_uri]
  api_client_id                   = module.api_local.client_id
  api_scope_id                    = module.api_local.scope_id
  api_service_principal_object_id = module.api_local.service_principal_object_id
}

module "spa_dev" {
  source = "./modules/spa-registration"

  display_name                    = "money-bae-dev"
  redirect_uris                   = [var.dev_redirect_uri, local.postman_callback_redirect_uri]
  api_client_id                   = module.api_dev.client_id
  api_scope_id                    = module.api_dev.scope_id
  api_service_principal_object_id = module.api_dev.service_principal_object_id
}
```

- [ ] **Step 4: Write `outputs.tf`**

```hcl
output "spa_client_id_local" {
  value = module.spa_local.client_id
}

output "spa_client_id_dev" {
  value = module.spa_dev.client_id
}

output "api_client_id_local" {
  value = module.api_local.client_id
}

output "api_client_id_dev" {
  value = module.api_dev.client_id
}

output "api_identifier_uri_local" {
  value = module.api_local.identifier_uri
}

output "api_identifier_uri_dev" {
  value = module.api_dev.identifier_uri
}
```

- [ ] **Step 5: Write `environments/shared.tfvars`**

```hcl
local_redirect_uri = "http://localhost:3000"
# dev_redirect_uri intentionally omitted here — its default in variables.tf
# is a clearly-invalid placeholder domain. Override it with -var once the
# real CloudFront domain is known (see Task 4's runbook).
```

- [ ] **Step 6: Format, initialize against the real backend, and validate**

Run from `platform/entra-external-id/app-registrations/`:

```bash
terraform fmt -check -recursive
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=identity/app-registrations.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
terraform validate
```

Real backend, not `-backend=false` — same shared storage account as `tenant/` and `platform/db`, different `key`. See Task 1 Step 6 for why this is safe to run unsupervised (it only touches Terraform's own state-tracking metadata, never an Azure/Entra resource) and what to do if it fails on authorization.

Expected: `fmt -check -recursive` clean across root + both modules; `init` resolves the local module sources, downloads providers, and reports a successfully configured backend; `validate` prints `Success! The configuration is valid.` (validate succeeds even though `ciam_tenant_id` has no default — Terraform only checks config validity, not that every variable has a value.)

- [ ] **Step 7: Commit**

```bash
git add platform/entra-external-id/app-registrations/providers.tf platform/entra-external-id/app-registrations/variables.tf platform/entra-external-id/app-registrations/main.tf platform/entra-external-id/app-registrations/outputs.tf platform/entra-external-id/app-registrations/environments/
git commit -m "Add Terraform root module for the four app registrations"
```

---

## Task 4: `platform/entra-external-id/CLAUDE.md` — runbook and scope doc

**Files:**
- Create: `platform/entra-external-id/CLAUDE.md`

**Interfaces:**
- Consumes: outputs defined in Tasks 1 and 3 (referenced by name in the runbook commands below).
- Produces: nothing consumed by later tasks — this is the operator-facing entry point, mirroring `platform/db/CLAUDE.md`.

- [ ] **Step 1: Write `CLAUDE.md`**

```markdown
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this directory.

## Project Overview

`platform/entra-external-id` is the Terraform (IaC) for money-bae's Microsoft
Entra External ID (CIAM) tenant and its app registrations. One tenant is
shared across environments — mirroring how `../db` shares one Postgres
server across two databases — holding four app registrations:

- `money-bae-local` / `money-bae-dev` — SPA (PKCE) clients for `../../clients/web`
- `money-bae-api-local` / `money-bae-api-dev` — API resources for `../../servers/api`,
  each exposing an `access_as_user` scope that its matching SPA app is
  pre-consented for.

Design spec: `../../docs/superpowers/specs/2026-09-04-entra-external-id-design.md`

## ⚠️ This Is Live Infrastructure

Real Azure/Entra resources. Never run `terraform apply` or `terraform
destroy` without explicit human confirmation of the `terraform plan`
output first. `terraform plan`/`validate`/`fmt`/`state list` are safe to
run freely.

## Two separate Terraform states, applied in order

`tenant/` and `app-registrations/` are independent root modules with
independent state, because the `azuread` provider needs the new tenant's
ID before it can do anything — which doesn't exist until `tenant/` has
already been applied.

### 1. Create the tenant

```bash
cd tenant
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=identity/tenant.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
terraform plan -var-file="environments/shared.tfvars"
# review the plan carefully, then:
terraform apply -var-file="environments/shared.tfvars"
```

### 2. Re-authenticate against the new tenant

The account that creates a CIAM tenant automatically becomes its Global
Administrator, but your Azure CLI session is still authenticated against
your normal subscription tenant. CIAM tenants have no subscription
attached, so:

```bash
NEW_TENANT_ID=$(terraform -chdir=tenant output -raw tenant_id)
az login --tenant "$NEW_TENANT_ID" --allow-no-subscriptions
```

### 3. Create the app registrations

```bash
cd app-registrations
terraform init \
  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
  -backend-config="storage_account_name=stmbtfstateshared" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=identity/app-registrations.tfstate" \
  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
terraform plan \
  -var="ciam_tenant_id=$NEW_TENANT_ID" \
  -var-file="environments/shared.tfvars"
# review the plan carefully, then:
terraform apply \
  -var="ciam_tenant_id=$NEW_TENANT_ID" \
  -var-file="environments/shared.tfvars"
```

`dev_redirect_uri` defaults to a placeholder domain (see
`app-registrations/variables.tf`) until `platform/web-client` has been
deployed at least once. Once you know the real CloudFront domain, add
`-var="dev_redirect_uri=https://<real-domain>.cloudfront.net"` to the plan
and apply commands above (or re-apply later to update it — the SPA app
registration's redirect URI can be changed after the fact).

### 4. Wire up the Postman collection

Once `app-registrations/` is applied, fill in the OAuth variables in
`../../servers/api/docs/collections/money-bae-api-local.postman_environment.json`
and `money-bae-api-dev.postman_environment.json` using:

```bash
terraform -chdir=app-registrations output -raw spa_client_id_local
terraform -chdir=app-registrations output -raw api_identifier_uri_local
terraform -chdir=tenant output -raw tenant_domain
```

`authUrl`/`tokenUrl` follow the pattern
`https://<ciam_tenant_name>.ciamlogin.com/<tenant_domain>/oauth2/v2.0/{authorize,token}`
— e.g. for this tenant:
`https://kkbaeexternalid.ciamlogin.com/kkbaeexternalid.onmicrosoft.com/oauth2/v2.0/authorize`.
`scope` is `<api_identifier_uri>/access_as_user`.

## Deferred (not covered by this Terraform)

- The CIAM user flow (sign-up/sign-in experience) — no Terraform
  resource for this yet in `azuread` or `azapi` (ARM-only); it's a
  Microsoft Graph `identity/` concept. Configure manually in the Portal
  for now.
- Identity provider config — local-account policy and social logins
  (Google, Facebook, Apple) are Microsoft Graph
  `identity/identityProviders` (`socialIdentityProvider`) resources,
  same gap as user flows.
- See the GitHub issue tracking research into automating both (filed
  after this Terraform ships).
```

- [ ] **Step 2: Commit**

```bash
git add platform/entra-external-id/CLAUDE.md
git commit -m "Add CLAUDE.md runbook for platform/entra-external-id"
```

---

## Task 5: Postman — OAuth2 PKCE auth on the collection

**Files:**
- Modify: `servers/api/docs/collections/money-bae-api.postman_collection.json`
- Modify: `servers/api/docs/collections/money-bae-api-local.postman_environment.json`
- Modify: `servers/api/docs/collections/money-bae-api-dev.postman_environment.json`

**Interfaces:**
- Consumes: the *names* of Task 3's Terraform outputs (`spa_client_id_local`/`spa_client_id_dev`, `api_identifier_uri_local`/`api_identifier_uri_dev`) and Task 1's `tenant_domain` — referenced in variable descriptions, not resolved to real values by this task (no `apply` has run).

- [ ] **Step 1: Read the current collection file**

Confirm current structure before editing (already known from investigation, but re-read to get exact current byte content for the edit):

```bash
cat servers/api/docs/collections/money-bae-api.postman_collection.json
```

- [ ] **Step 2: Add collection-level OAuth2 auth**

Edit `servers/api/docs/collections/money-bae-api.postman_collection.json`: add an `"auth"` key as a sibling of `"info"` (top level of the collection object), and update the `token` variable's description since it's superseded by the new `auth` block for interactive use (leave the variable itself in place — some requests may still reference `{{token}}` directly for scripted/CI use):

```json
"auth": {
  "type": "oauth2",
  "oauth2": [
    { "key": "grant_type", "value": "authorization_code_with_pkce", "type": "string" },
    { "key": "authUrl", "value": "{{authUrl}}", "type": "string" },
    { "key": "accessTokenUrl", "value": "{{tokenUrl}}", "type": "string" },
    { "key": "clientId", "value": "{{clientId}}", "type": "string" },
    { "key": "scope", "value": "{{scope}}", "type": "string" },
    { "key": "redirect_uri", "value": "https://oauth.pstmn.io/v1/callback", "type": "string" },
    { "key": "client_authentication", "value": "header", "type": "string" },
    { "key": "addTokenTo", "value": "header", "type": "string" },
    { "key": "tokenName", "value": "money-bae-token", "type": "string" }
  ]
}
```

Update the `token` variable's `description` in the `"variable"` array to:
`"Manually-pasted fallback bearer token. Prefer the collection's OAuth 2.0 auth (Get New Access Token in Postman) once a real IdP is wired in — see platform/entra-external-id/CLAUDE.md."`

- [ ] **Step 3: Add OAuth variables to the local environment**

Edit `servers/api/docs/collections/money-bae-api-local.postman_environment.json`, appending to the `"values"` array:

```json
{
  "key": "clientId",
  "value": "",
  "type": "default",
  "enabled": true
},
{
  "key": "authUrl",
  "value": "",
  "type": "default",
  "enabled": true
},
{
  "key": "tokenUrl",
  "value": "",
  "type": "default",
  "enabled": true
},
{
  "key": "scope",
  "value": "",
  "type": "default",
  "enabled": true
}
```

(Values stay empty here — Postman environment JSON has no `description`
field on `values` entries the way the collection's top-level `variable`
array does. Document where to get the real values in Step 5 instead.)

- [ ] **Step 4: Add the same four OAuth variables to the dev environment**

Same four entries (same keys, empty values) appended to `"values"` in
`servers/api/docs/collections/money-bae-api-dev.postman_environment.json`.

- [ ] **Step 5: Document how to fill in the real values**

Add a line to `servers/api/docs/collections/` — check whether a README
already exists there first:

```bash
ls servers/api/docs/collections/
```

If no README exists, create `servers/api/docs/collections/README.md`:

```markdown
# Postman collections

`money-bae-api.postman_collection.json` — import this collection.
`money-bae-api-local.postman_environment.json` / `money-bae-api-dev.postman_environment.json` — import both environments, select one at a time.

## OAuth setup (one-time, per environment)

After `platform/entra-external-id/app-registrations` has been applied,
fill in each environment's `clientId`/`authUrl`/`tokenUrl`/`scope`
variables:

- `clientId` — `terraform -chdir=platform/entra-external-id/app-registrations output -raw spa_client_id_local` (or `_dev`)
- `authUrl` — `https://kkbaeexternalid.ciamlogin.com/kkbaeexternalid.onmicrosoft.com/oauth2/v2.0/authorize`
- `tokenUrl` — `https://kkbaeexternalid.ciamlogin.com/kkbaeexternalid.onmicrosoft.com/oauth2/v2.0/token`
- `scope` — `terraform -chdir=platform/entra-external-id/app-registrations output -raw api_identifier_uri_local` (or `_dev`) with `/access_as_user` appended

Then in Postman, on a request using this collection's auth, click
"Get New Access Token" — it uses Authorization Code with PKCE, no
client secret, since these are SPA (public client) app registrations.
```

If a README already exists there, add this section to it instead of
overwriting the rest of the file.

- [ ] **Step 6: Validate the edited JSON files stay well-formed**

```bash
python3 -m json.tool servers/api/docs/collections/money-bae-api.postman_collection.json > /dev/null
python3 -m json.tool servers/api/docs/collections/money-bae-api-local.postman_environment.json > /dev/null
python3 -m json.tool servers/api/docs/collections/money-bae-api-dev.postman_environment.json > /dev/null
echo "all valid JSON"
```

Expected: `all valid JSON` with no errors from any of the three `json.tool` calls.

- [ ] **Step 7: Commit**

```bash
git add servers/api/docs/collections/
git commit -m "Configure Postman collection for OAuth2 PKCE against the new SPA app registrations"
```
