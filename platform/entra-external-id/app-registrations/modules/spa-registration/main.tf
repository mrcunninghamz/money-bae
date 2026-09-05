terraform {
  required_providers {
    azuread = {
      source  = "hashicorp/azuread"
      version = "=3.9.0"
    }
  }
}

resource "azuread_application" "spa" {
  display_name     = var.display_name
  sign_in_audience = "AzureADMyOrg"

  # CIAM rejects application creation with AccessTokenAcceptedVersion 1 or
  # null ("InvalidAccessTokenVersion") — every app in the tenant needs v2
  # tokens, not just the API resource.
  api {
    requested_access_token_version = 2
  }

  single_page_application {
    redirect_uris = var.redirect_uris
  }

  # given_name/family_name aren't in this tenant's default
  # claims_supported (confirmed against the real discovery document), but
  # that only reflects what's returned via scopes -- optional_claims is a
  # separate mechanism (see api-registration's email optional claim for
  # why: it needed this same treatment to show up in the access token).
  # Worth trying explicitly rather than assuming claims_supported is
  # exhaustive for what optional_claims can add.
  optional_claims {
    id_token {
      name = "given_name"
    }
    id_token {
      name = "family_name"
    }
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
