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
