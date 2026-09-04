# Entra External ID — Design Spec

## Purpose

money-bae has no real identity provider yet: `servers/api/internal/auth`
defines `OIDCVerifier` but it's unused (`MockVerifier` is wired in, per
`servers/api/CLAUDE.md`), and `clients/web` has a `login.tsx` route with no
OAuth wiring. This pass provisions the identity infrastructure — a
Microsoft Entra External ID (CIAM) tenant plus the app registrations
needed for a SPA + API OAuth flow — as Terraform, following the patterns
established in `platform/db/infrastructure/`. It does not wire the SPA or
API code to the new IdP; that's a follow-up once the infra exists.

## Scope of this pass

In scope:
- One Entra External ID (CIAM) tenant, created via Terraform.
- Four app registrations: `money-bae-local`, `money-bae-dev` (SPA/PKCE
  clients) and `money-bae-api-local`, `money-bae-api-dev` (API resources),
  one tenant shared across both environments.
- Each API app exposes a scope; each SPA app is granted (and
  admin-consented for) that scope — a working token-acquisition path, not
  just empty registrations.
- Redirect URIs: `local` → the web client's local dev server origin;
  `dev` → the web client's deployed CloudFront origin.
- Updating the Postman collection/environments to use OAuth2
  (Authorization Code + PKCE) with the SPA client ID instead of the
  current `mock-token` placeholder.

Deferred / out of scope (see "Deferred" section):
- Provisioning the CIAM user flow (sign-up/sign-in experience).
- Configuring identity providers — local-account policy, and social
  logins (Google, Facebook, Apple).
- Wiring `clients/web` or `servers/api` to actually use the new IdP
  (MSAL integration, switching `OIDC_ISSUER_URL`/`OIDC_AUDIENCE`,
  swapping `MockVerifier` for `OIDCVerifier`).
- A custom domain for the web client (dev redirect URI uses the raw
  CloudFront domain for now).

## Why two Terraform root modules, not one

Creating the tenant and creating app registrations inside it require two
different auth contexts:

- Creating the CIAM tenant is a subscription-scoped operation — same
  ambient `az login` auth `platform/db` already uses against subscription
  `c6f1212c-ec19-425f-96a0-41f2db717ea8`.
- Managing app registrations requires the `azuread` provider's `tenant_id`
  to point at the *new* tenant, and requires the calling identity to be
  authenticated *against that tenant* specifically (CIAM tenants have no
  Azure subscription attached, so `az login --tenant <id>
  --allow-no-subscriptions` is required after the tenant exists). The
  account that creates a CIAM tenant automatically becomes its Global
  Administrator, so no separate app-registration bootstrapping is needed
  beyond that re-login.

Terraform forbids a provider's configuration from depending on a resource
created in the same apply (the `tenant_id` isn't known until after stage
1 applies), so this has to be two independent root modules with two state
files, applied in order, with a manual re-login step in between.

## Repo layout

Mirrors `platform/api/` and `platform/web-client/` — no `infrastructure/`
wrapper subfolder (that's specific to `platform/db/`, an older convention
that wasn't carried forward to the newer platform projects):

```
platform/entra-external-id/
├── CLAUDE.md
├── tenant/                          # stage 1 — subscription-scoped auth
│   ├── main.tf                      # resource group + azapi_resource (ciamDirectories)
│   ├── providers.tf                 # azurerm + azapi, backend azurerm (shared state account)
│   ├── variables.tf
│   ├── outputs.tf                   # tenant_id, tenant_domain
│   └── environments/
│       └── shared.tfvars            # one tenant, not per-environment
└── app-registrations/               # stage 2 — tenant-scoped auth
    ├── main.tf                      # module "app_pair" x2 (local, dev)
    ├── providers.tf                 # azuread provider, tenant_id = var.ciam_tenant_id
    ├── variables.tf
    ├── outputs.tf                   # SPA + API client IDs, API Application ID URIs, per env
    ├── modules/
    │   └── app-registration-pair/   # one SPA app + one API app + scope + permission grant
    │       ├── main.tf
    │       ├── variables.tf
    │       └── outputs.tf
    └── environments/
        ├── local.tfvars
        └── dev.tfvars
```

State: same shared backend as `platform/db` (`stmbtfstateshared` storage
account, `rg-moneybae-tfstate-shared` resource group, `tfstate`
container), new keys `identity/tenant.tfstate` and
`identity/app-registrations.tfstate`.

## `tenant/` — CIAM tenant creation

```hcl
# providers.tf
terraform {
  required_version = ">= 1.1"
  required_providers {
    azurerm = { source = "hashicorp/azurerm", version = "=4.1.0" }
    azapi   = { source = "azure/azapi", version = "~> 1.14" }
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

```hcl
# main.tf
resource "azurerm_resource_group" "identity" {
  name     = "rg-${var.app_short_name}-identity-${var.location_abrv}-shared"
  location = var.location
  tags     = var.tags
}

resource "azapi_resource" "ciam_tenant" {
  type      = "Microsoft.AzureActiveDirectory/ciamDirectories@2023-05-17-preview"
  name      = "moneybaeexternalid"   # <=26 chars, alphanumeric only, globally unique
  parent_id = azurerm_resource_group.identity.id
  location  = var.ciam_data_location  # "United States" | "Europe" | "Asia Pacific" | "Australia"
  tags      = var.tags

  body = {
    properties = {
      createTenantProperties = {
        countryCode = var.ciam_country_code   # e.g. "US"
        displayName = "Money Bae"
      }
    }
    sku = {
      name = "Standard"
      tier = "A0"
    }
  }
}
```

Confirmed against Microsoft's ARM/AzAPI schema reference for
`Microsoft.AzureActiveDirectory/ciamDirectories@2023-05-17-preview` — this
is the exact property shape, not a guess.

Outputs: `tenant_id` (from the resource's `output.tenantId`, exposed
through the azapi resource's `output` attribute) and the tenant's default
domain (`<name>.onmicrosoft.com` / `<name>.ciamlogin.com`).

## `app-registrations/` — the `app-registration-pair` module

Instantiated twice (`local`, `dev`), each taking `env_name` and
`redirect_uri` as inputs:

- **API app** (`money-bae-api-<env>`): `azuread_application` with
  `sign_in_audience = "AzureADMyOrg"`, an `identifier_uris` entry
  (`api://money-bae-api-<env>`), and one `api.oauth2_permission_scope`
  block (`access_as_user`, admin+user consent). Paired
  `azuread_service_principal`.
- **SPA app** (`money-bae-<env>`): `azuread_application` with a
  `single_page_application { redirect_uris = [var.redirect_uri] }` block
  (PKCE flow — not the `web` platform block, which is for
  confidential/server-side clients) and `required_resource_access`
  pointing at the API app's `access_as_user` scope. Paired
  `azuread_service_principal`.
- An `azuread_service_principal_delegated_permission_grant` pre-consenting
  the SPA's access to `access_as_user`, so signed-in users aren't prompted
  for admin consent.

Module outputs: SPA client ID, API client ID, API `identifier_uris[0]`
(this becomes the Go API's future `OIDC_AUDIENCE`).

Redirect URIs (`environments/{local,dev}.tfvars`):
- `local` → `http://localhost:3000` (matches `clients/web`'s `npm run dev`
  port per its `CLAUDE.md`).
- `dev` → the web client's CloudFront domain, hardcoded once known.
  `platform/web-client`'s distribution has no custom domain today — its
  domain is only known after `cdk deploy` runs. Fill this in manually
  after that first deploy, the same way `platform/db` pins `zone = "1"`
  to match live reality rather than computing it. Needs a manual update
  if the distribution is ever replaced.

## Deferred / out of scope (recap)

- **CIAM user flow** (sign-up/sign-in experience) — no first-class
  Terraform resource in `azuread` or `azapi` (ARM-only) for this; it's a
  Microsoft Graph `identity/` concept, still largely portal-managed.
- **Identity provider configuration** — local-account policy and social
  logins (Google, Facebook, Apple) are Microsoft Graph
  `identity/identityProviders` (`socialIdentityProvider`) resources, same
  automation gap as user flows.
- Action: after this pass ships, open a GitHub issue to research the
  right mechanism for both (direct Graph API calls, a generic
  Graph-wrapping Terraform provider, or accepting manual Portal steps)
  before attempting either as code.
- Wiring `clients/web`/`servers/api` to the new IdP is a separate,
  later piece of work.

## Postman updates (final step of this pass)

`servers/api/docs/collections/money-bae-api.postman_collection.json`
currently authenticates with a raw `{{token}}` variable defaulted to
`mock-token` (its own description already says "swap for a real bearer
token once an IdP is configured" — that moment is now, for local
testing purposes even though `MockVerifier` is still what's wired into
the API):

- Change the collection's Authorization to OAuth 2.0, Authorization Code
  with PKCE, no client secret (the SPA app registrations are public
  clients).
- Add `clientId`, `authUrl`, `tokenUrl`, and `scope` variables to both
  `money-bae-api-local.postman_environment.json` and
  `money-bae-api-dev.postman_environment.json`, sourced from this pass's
  Terraform outputs (`money-bae-local`/`money-bae-dev` SPA client IDs,
  the CIAM tenant's authorize/token endpoints, and
  `api://money-bae-api-<env>/access_as_user` as the scope).
- Per existing convention ([[feedback_postman_dev_env_is_deployed_url]]),
  "Local" and "Dev" stay the only two environments — update them in
  place rather than adding new ones.
