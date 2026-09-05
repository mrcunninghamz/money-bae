# platform/db Subscription Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Stop after Task 7.** Every numbered task below only touches committed
> repo files (Terraform code, docs, example env templates) and read-only
> `terraform validate`/`fmt`/`state list`/grep commands. Nothing in Tasks
> 1–7 creates, modifies, or destroys a real Azure resource, and nothing
> copies real data. The **Manual Runbook** appendix after Task 7 documents
> the real Azure/data-migration commands — it is explicitly **not** a task
> for automated execution. A human runs it themselves, at their own pace,
> after Tasks 1–7 are merged.

**Goal:** Move `platform/db`'s Terraform code (and, via the runbook, its
Postgres server + Terraform state) from the old money-bae Azure
subscription to the `kmerecidolive` personal subscription already used by
`platform/entra-external-id` (#57), adopting `kkb` org-level naming, and
flatten the `infrastructure/` wrapper folder to match `platform/api`,
`platform/web-client`, and `platform/entra-external-id`.

**Architecture:** No application code changes. This is a Terraform
root-module edit (provider subscription + resource naming), a directory
flatten (git mv), and doc/template updates everywhere the old hostname or
subscription ID is hardcoded. The actual server creation and data copy
are real, human-run Azure operations captured as a runbook, not code.

**Tech Stack:** Terraform >= 1.1, `azurerm` provider `=4.1.0`, Azure CLI,
`pg_dump`/`pg_restore`, Go (`servers/api/cmd/import-legacy-data`).

**Spec:** `docs/superpowers/specs/2026-09-04-platform-db-migration-design.md`

## Global Constraints

- Terraform >= 1.1 (module uses a `moved` block) — unchanged requirement.
- **Never** run `terraform apply`/`terraform destroy` without a human
  reviewing `terraform plan` output first. `terraform plan`/`validate`/
  `fmt`/`state list` are safe to run freely (`platform/db/CLAUDE.md`).
- `TF_VAR_money_bae_db_admin_password` is set via shell env only (e.g.
  `~/.zshenv`) — never as a `-var` flag, never written to any file.
- New subscription: `085f952f-488d-4c4d-bd33-0fcf8fd37e17` (tenant
  `kmerecidolive.onmicrosoft.com`).
- New/target resource names: resource group `rg-kkb-core-cus-dev`, server
  `psql-kkb-core-cus-dev`. Databases stay `money_bae`/`money_bae_api`;
  admin login stays `mbae`.
- Terraform state backend (already provisioned for #57 — do not create):
  storage account `stkkbtfstatecus`, resource group
  `rg-kkbae-tfstate-shared`, container `tfstate`. This plan's key:
  `db/dev.cus.tfstate`.
- The old server (`psql-mb-core-cus-dev`) and old state backend
  (`stmbtfstateshared`/`rg-moneybae-tfstate-shared`, key
  `core/dev.cus.tfstate`) are left completely untouched — decommissioning
  them is out of scope for this plan.
- Every commit message references issue `#60`.

---

## Task 1: Flatten `platform/db/infrastructure/` into `platform/db/`

**Files:**
- Move: `platform/db/infrastructure/README.md` → `platform/db/README.md`
- Move: `platform/db/infrastructure/main.tf` → `platform/db/main.tf`
- Move: `platform/db/infrastructure/providers.tf` → `platform/db/providers.tf`
- Move: `platform/db/infrastructure/variables.tf` → `platform/db/variables.tf`
- Move: `platform/db/infrastructure/outputs.tf` → `platform/db/outputs.tf`
- Move: `platform/db/infrastructure/environments/` → `platform/db/environments/`
- Move: `platform/db/infrastructure/modules/` → `platform/db/modules/`

**Interfaces:**
- Produces: from this task on, every later task in this plan refers to
  these files at their new top-level paths (e.g. `platform/db/providers.tf`,
  not `platform/db/infrastructure/providers.tf`).

- [ ] **Step 1: Move the files**

```bash
cd platform/db
git mv infrastructure/README.md README.md
git mv infrastructure/main.tf main.tf
git mv infrastructure/providers.tf providers.tf
git mv infrastructure/variables.tf variables.tf
git mv infrastructure/outputs.tf outputs.tf
git mv infrastructure/environments environments
git mv infrastructure/modules modules
```

- [ ] **Step 2: Confirm the `infrastructure/` directory is gone**

Run: `ls platform/db/`
Expected: `README.md  environments  main.tf  modules  outputs.tf  providers.tf  variables.tf  CLAUDE.md  AGENTS.md` — no `infrastructure/` entry.

- [ ] **Step 3: Verify the moved config is still internally consistent**

The module source in `main.tf` (`source = "./modules/postgresql"`) is a
relative path — since `main.tf` and `modules/` moved together, it stays
valid. Confirm with:

```bash
cd platform/db
terraform init -backend=false
terraform validate
```

Expected: `Success! The configuration is valid.`

- [ ] **Step 4: Commit**

```bash
git add platform/db
git commit -m "Flatten platform/db/infrastructure/ into platform/db/ (#60)"
```

---

## Task 2: Point Terraform at the new subscription and `kkb` naming

**Files:**
- Modify: `platform/db/providers.tf`
- Modify: `platform/db/environments/dev.cus.tfvars`

**Interfaces:**
- Consumes: files at their post-flatten paths from Task 1.
- Produces: `terraform plan`/`apply` (run later, by a human, per the
  runbook) will target subscription `085f952f-488d-4c4d-bd33-0fcf8fd37e17`
  and compute resource names `rg-kkb-core-cus-dev`/`psql-kkb-core-cus-dev`.

- [ ] **Step 1: Update the provider subscription ID**

```diff
--- a/platform/db/providers.tf
+++ b/platform/db/providers.tf
@@
 provider "azurerm" {
-  subscription_id = "c6f1212c-ec19-425f-96a0-41f2db717ea8"
+  subscription_id = "085f952f-488d-4c4d-bd33-0fcf8fd37e17"
   features {}
   resource_provider_registrations = "none"
 }
```

- [ ] **Step 2: Update `app_short_name` to `kkb`**

```diff
--- a/platform/db/environments/dev.cus.tfvars
+++ b/platform/db/environments/dev.cus.tfvars
@@
 environment              = "dev"
 location                 = "centralus"
 location_abrv            = "cus"
-app_short_name           = "mb"
+app_short_name           = "kkb"
 component                = "core"
 db_allow_public_access   = true
 money_bae_db_admin_login = "mbae"
```

This changes the computed resource group name to `rg-kkb-core-cus-dev`
(`main.tf:1-2`) and server name to `psql-kkb-core-cus-dev` (`main.tf:10`).
The `money_bae`/`money_bae_api` database names (`main.tf:13`) and the
`mbae` admin login are untouched — they're app-specific, not
org-naming-specific.

- [ ] **Step 3: Validate**

```bash
cd platform/db
terraform validate
terraform fmt -check -recursive
```

Expected: `Success! The configuration is valid.` and no output from `fmt -check` (already formatted).

- [ ] **Step 4: Confirm the plan would target the new names (no backend needed for this check)**

```bash
cd platform/db
terraform plan -var-file="environments/dev.cus.tfvars" -out=/dev/null 2>&1 | grep -E "rg-kkb-core-cus-dev|psql-kkb-core-cus-dev" | head -5
```

Expected: both new names appear (this plan runs against the local/no-op
backend from `-backend=false` in Task 1, so it will show a plan to create
everything from scratch — that's expected and not applied).

- [ ] **Step 5: Commit**

```bash
git add platform/db/providers.tf platform/db/environments/dev.cus.tfvars
git commit -m "Point platform/db at the kmerecidolive subscription and kkb naming (#60)"
```

---

## Task 3: Update `platform/db/README.md`

**Files:**
- Modify: `platform/db/README.md`

- [ ] **Step 1: Update the architecture section's resource names**

```diff
--- a/platform/db/README.md
+++ b/platform/db/README.md
@@
 ## Architecture

-- **Resource Group**: `rg-mb-core-cus-dev`
-- **PostgreSQL Flexible Server**: `psql-mb-core-cus-dev`, holding two databases:
+- **Resource Group**: `rg-kkb-core-cus-dev`
+- **PostgreSQL Flexible Server**: `psql-kkb-core-cus-dev`, holding two databases:
   - `money_bae` — the Rust TUI application's data
   - `money_bae_api` — the Go API's data (independent schema, same server)
```

- [ ] **Step 2: Update the prerequisites' backend note**

```diff
@@
 - Azure CLI authenticated (`az login`)
 - Terraform >= 1.1 (required for the `moved` block used in `modules/postgresql/main.tf`)
 - Azure subscription with permissions to create resources
-- Remote state storage account (already provisioned: `stmbtfstateshared`)
+- Remote state storage account (already provisioned: `stkkbtfstatecus`,
+  the same kkb-org company-level backend used by `platform/entra-external-id`)
```

- [ ] **Step 3: Update the "Local Deployment" intro and backend-init command**

```diff
@@
 ## Local Deployment

-All commands below assume you're in this directory (`platform/db/infrastructure/`).
+All commands below assume you're in this directory (`platform/db/`).

 ### 1. Initialize Backend

 Configure remote state storage:

 ```bash
 terraform init \
-  -backend-config="resource_group_name=rg-moneybae-tfstate-shared" \
-  -backend-config="storage_account_name=stmbtfstateshared" \
+  -backend-config="resource_group_name=rg-kkbae-tfstate-shared" \
+  -backend-config="storage_account_name=stkkbtfstatecus" \
   -backend-config="container_name=tfstate" \
-  -backend-config="key=core/dev.cus.tfstate" \
-  -backend-config="subscription_id=c6f1212c-ec19-425f-96a0-41f2db717ea8"
+  -backend-config="key=db/dev.cus.tfstate" \
+  -backend-config="subscription_id=085f952f-488d-4c4d-bd33-0fcf8fd37e17"
 ```
```

- [ ] **Step 4: Update the Module Structure diagram**

```diff
@@
 ## Module Structure

 ```
-infrastructure/
+platform/db/
 ├── main.tf              # Main orchestration
 ├── providers.tf         # Azure provider config
 ├── variables.tf         # Input variables
 ├── outputs.tf           # Output values
 ├── environments/        # Environment-specific tfvars
 │   └── dev.cus.tfvars
 └── modules/             # Reusable modules
     └── postgresql/      # PostgreSQL Flexible Server module
 ```
```

- [ ] **Step 5: Verify no stale references remain**

```bash
grep -n "c6f1212c-ec19-425f-96a0-41f2db717ea8\|rg-mb-core-cus-dev\|psql-mb-core-cus-dev\|stmbtfstateshared\|rg-moneybae-tfstate-shared\|infrastructure/" platform/db/README.md
```

Expected: no output.

- [ ] **Step 6: Commit**

```bash
git add platform/db/README.md
git commit -m "Update platform/db/README.md for the subscription move (#60)"
```

---

## Task 4: Update `platform/db/CLAUDE.md` and `platform/db/AGENTS.md`

These two files are byte-identical (repo convention — see `tui/CLAUDE.md`/
`tui/AGENTS.md` and `servers/api/CLAUDE.md`/`AGENTS.md`). Apply the same
edit to both.

**Files:**
- Modify: `platform/db/CLAUDE.md`
- Modify: `platform/db/AGENTS.md`

- [ ] **Step 1: Rewrite both files to this exact content**

```markdown
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
```

- [ ] **Step 2: Copy the same content into AGENTS.md**

```bash
cp platform/db/CLAUDE.md platform/db/AGENTS.md
diff platform/db/CLAUDE.md platform/db/AGENTS.md
```

Expected: `diff` produces no output (files identical).

- [ ] **Step 3: Verify no stale references remain**

```bash
grep -n "c6f1212c-ec19-425f-96a0-41f2db717ea8\|psql-mb-core-cus-dev\|stmbtfstateshared\|rg-moneybae-tfstate-shared\|infrastructure/" platform/db/CLAUDE.md platform/db/AGENTS.md
```

Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add platform/db/CLAUDE.md platform/db/AGENTS.md
git commit -m "Update platform/db CLAUDE.md/AGENTS.md for the subscription move (#60)"
```

---

## Task 5: Update `servers/api/.env.dev.example`

**Files:**
- Modify: `servers/api/.env.dev.example`

- [ ] **Step 1: Update the hostname**

```diff
--- a/servers/api/.env.dev.example
+++ b/servers/api/.env.dev.example
@@
 # Copy to .env.dev to point at the shared Azure dev database, then run
 # ./use-dev-env.sh to activate it.
-DATABASE_URL=postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae_api?sslmode=require
+DATABASE_URL=postgres://username:password@psql-kkb-core-cus-dev.postgres.database.azure.com/money_bae_api?sslmode=require
 PORT=8080
```

- [ ] **Step 2: Verify**

```bash
grep -n "psql-mb-core-cus-dev" servers/api/.env.dev.example
```

Expected: no output.

- [ ] **Step 3: Commit**

```bash
git add servers/api/.env.dev.example
git commit -m "Update servers/api dev env template hostname for the subscription move (#60)"
```

---

## Task 6: Update `tui/CLAUDE.md`, `tui/AGENTS.md`, `tui/README.md`

`tui/CLAUDE.md` and `tui/AGENTS.md` are byte-identical (same convention as
Task 4). Each has one occurrence of the old hostname in an example
connection string comment; `tui/README.md` has two.

**Files:**
- Modify: `tui/CLAUDE.md`
- Modify: `tui/AGENTS.md`
- Modify: `tui/README.md`

- [ ] **Step 1: Update `tui/CLAUDE.md`**

```diff
--- a/tui/CLAUDE.md
+++ b/tui/CLAUDE.md
@@
 # For Azure PostgreSQL (from Terraform outputs):
-# DATABASE_URL=postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require
+# DATABASE_URL=postgres://username:password@psql-kkb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require
```

- [ ] **Step 2: Apply the identical change to `tui/AGENTS.md`**

`tui/CLAUDE.md` and `tui/AGENTS.md` are byte-identical before this task
(confirmed convention). Rather than re-typing the same one-line edit
twice, overwrite `AGENTS.md` with the just-edited `CLAUDE.md` content and
verify:

```bash
cp tui/CLAUDE.md tui/AGENTS.md
diff tui/CLAUDE.md tui/AGENTS.md
```

Expected: `diff` produces no output (files identical).

- [ ] **Step 3: Update `tui/README.md`'s two occurrences**

```diff
--- a/tui/README.md
+++ b/tui/README.md
@@
 # For Azure PostgreSQL, use connection strings from Terraform outputs
-# Example: postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require
+# Example: postgres://username:password@psql-kkb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require
@@
 # For local: database_connection_string = "postgres://username@localhost/money_bae_dev"
-# For Azure: database_connection_string = "postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require"
+# For Azure: database_connection_string = "postgres://username:password@psql-kkb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require"
```

- [ ] **Step 4: Verify**

```bash
grep -rn "psql-mb-core-cus-dev" tui/CLAUDE.md tui/AGENTS.md tui/README.md
```

Expected: no output.

- [ ] **Step 5: Commit**

```bash
git add tui/CLAUDE.md tui/AGENTS.md tui/README.md
git commit -m "Update tui docs' example hostname for the subscription move (#60)"
```

---

## Task 7: Repo-wide sweep for stale references

**Files:** none modified — this is a verification-only task. If it finds
anything, go back and fix it in the relevant task above, then re-run.

- [ ] **Step 1: Search the whole repo for the old subscription ID and old resource names**

```bash
grep -rn "c6f1212c-ec19-425f-96a0-41f2db717ea8\|psql-mb-core-cus-dev\|rg-mb-core-cus-dev\|stmbtfstateshared" \
  --include="*.md" --include="*.tf" --include="*.tfvars" --include="*.example" \
  --exclude-dir=".terraform" .
```

Expected: the **only** remaining hits are in the dated historical
planning docs `docs/superpowers/plans/2026-09-02-money-bae-api.md` and
`docs/superpowers/specs/2026-09-02-money-bae-api-design.md` (these are a
historical record of past decisions, not living documentation — like git
history, they're intentionally left alone). If any hit appears outside
those two files, fix it before proceeding.

- [ ] **Step 2: Confirm `platform/db` has no leftover `infrastructure/` directory**

```bash
find platform/db -type d -name infrastructure
```

Expected: no output.

- [ ] **Step 3: Final validate**

```bash
cd platform/db
terraform validate
```

Expected: `Success! The configuration is valid.`

(No commit for this task — it's verification-only. If Step 1 or 2 found
something, the fix was already committed as part of amending the
appropriate earlier task.)

---

## Manual Runbook (human-executed — NOT part of automated task execution)

Everything below creates or modifies real Azure resources, or copies real
production data. Per the Global Constraints above and the project's
"never apply/destroy without human confirmation" rule, **do not run these
as part of automated plan execution.** This section is for you (or
whoever is at the keyboard) to work through manually, at your own pace,
after Tasks 1–7 above are merged. Each step includes the exact command;
review the output before moving to the next step.

### A. Initialize Terraform against the new backend key

```bash
cd platform/db
terraform init \
  -backend-config="resource_group_name=rg-kkbae-tfstate-shared" \
  -backend-config="storage_account_name=stkkbtfstatecus" \
  -backend-config="container_name=tfstate" \
  -backend-config="key=db/dev.cus.tfstate" \
  -backend-config="subscription_id=085f952f-488d-4c4d-bd33-0fcf8fd37e17" \
  -reconfigure
```

`-reconfigure` (not `-migrate-state`) is deliberate: `db/dev.cus.tfstate`
is a brand-new, empty key. There is nothing to migrate — the new server is
a distinct resource in a different tenant, so Terraform starts from empty
state and creates everything fresh. The old backend/key
(`stmbtfstateshared`/`core/dev.cus.tfstate`) is never touched.

### B. Review the plan

```bash
terraform plan -var-file="environments/dev.cus.tfvars"
```

Confirm it shows **only creates** (resource group `rg-kkb-core-cus-dev`,
server `psql-kkb-core-cus-dev`, the two databases, and the firewall
rules) — nothing to destroy, since this is a fresh empty state.
`TF_VAR_money_bae_db_admin_password` must already be set in your shell
environment (see Global Constraints).

### C. Apply

```bash
terraform apply -var-file="environments/dev.cus.tfvars"
```

Only after reviewing and being satisfied with the plan from step B.

### D. Copy `money_bae` (tui data) from the old server to the new one

```bash
cd tui
./backup-db.sh prod   # dumps the OLD server's money_bae (reads DATABASE_URL from tui/.env.prod, which still points at the old server at this point)
pg_restore -d "<new-server-money_bae-connection-string>" money_bae_prod_<timestamp>.dump
```

Get `<new-server-money_bae-connection-string>` from
`terraform output -raw postgresql_connection_strings` in `platform/db/`
(it's a map keyed by database name — pull the `money_bae` entry).

### E. Regenerate `money_bae_api` on the new server

```bash
cd servers/api
export SOURCE_DATABASE_URL="<new-server-money_bae-connection-string>"   # same one used in step D, now populated
# Point DATABASE_URL at the NEW server's money_bae_api (edit .env.dev to the new host first, matching step F below), then:
./use-dev-env.sh
go run ./cmd/import-legacy-data
```

This matches issue #60's own guidance: reproducing `money_bae_api` via
the importer (rather than a byte-for-byte dump/restore) is equivalent
here because App Runner has never been deployed live, so `money_bae_api`
has no data beyond what the original importer run produced.

### F. Manual cutover (per-machine, not repo files)

- **tui**: edit `~/.config/money-bae/money-bae.toml`'s
  `database_connection_string`, and `tui/.env.prod`'s `DATABASE_URL`, to
  the new server's `money_bae` connection string.
- **servers/api**: edit local `servers/api/.env.dev`'s `DATABASE_URL` to
  the new server's `money_bae_api` connection string (needed before step
  E above, if not already done).
- **platform/api** (App Runner): check whether the Secrets Manager secret
  `money-bae-api/database-url` exists yet:

  ```bash
  aws secretsmanager describe-secret --secret-id money-bae-api/database-url
  ```

  If it exists, update its value to the new `money_bae_api` connection
  string. If it doesn't exist (likely, since `cdk deploy` has never been
  run for this stack), there's nothing to update yet.

### G. Done

At this point the new server is live with current data, and both apps'
local configs point at it. The old server and old state backend are
still running, untouched — decommissioning them is a separate follow-up,
not part of this plan.
