terraform {
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
}

resource "random_uuid" "access_as_user" {}

resource "azuread_application" "api" {
  display_name     = var.display_name
  sign_in_audience = "AzureADMyOrg"
  identifier_uris  = ["api://${var.display_name}"]

  api {
    # Also keeps identifier_uris legal: api://<string> is normally only
    # allowed when <string> is a GUID or verified domain, but apps with
    # requestedAccessTokenVersion = 2 are exempt from that restriction by
    # default. Removing this would break `terraform apply` non-obviously.
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

  # Access tokens for a custom API scope carry a minimal claim set by
  # default -- no email, regardless of which scopes the client requested
  # (openid/profile/email mainly affect the ID token, not this). email is
  # a built-in optional claim in CIAM, so no `source` here -- `source =
  # "user"` is only for custom extension properties, and is a documented
  # way to silently break this (the claim just doesn't show up).
  optional_claims {
    access_token {
      name = "email"
    }
  }
}

resource "azuread_service_principal" "api" {
  client_id = azuread_application.api.client_id
}
