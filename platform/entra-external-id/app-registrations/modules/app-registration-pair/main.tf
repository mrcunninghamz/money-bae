resource "random_uuid" "api_scope_access_as_user" {}

resource "azuread_application" "api" {
  display_name     = "money-bae-api-${var.env_name}"
  sign_in_audience = "AzureADMyOrg"
  identifier_uris  = ["api://money-bae-api-${var.env_name}"]

  api {
    requested_access_token_version = 2

    oauth2_permission_scope {
      id                         = random_uuid.api_scope_access_as_user.result
      value                      = "access_as_user"
      type                       = "User"
      enabled                    = true
      admin_consent_description  = "Allow the app to access money-bae-api-${var.env_name} on behalf of the signed-in user"
      admin_consent_display_name = "Access money-bae-api-${var.env_name}"
      user_consent_description   = "Allow the app to access money-bae-api-${var.env_name} on your behalf"
      user_consent_display_name  = "Access money-bae-api-${var.env_name}"
    }
  }
}

resource "azuread_service_principal" "api" {
  client_id = azuread_application.api.client_id
}

resource "azuread_application" "spa" {
  display_name     = "money-bae-${var.env_name}"
  sign_in_audience = "AzureADMyOrg"

  single_page_application {
    # oauth.pstmn.io is Postman's OAuth2 callback proxy, included so the
    # Postman collection (Task 5) can obtain real tokens for manual API
    # testing without a second app registration.
    redirect_uris = [var.redirect_uri, "https://oauth.pstmn.io/v1/callback"]
  }

  required_resource_access {
    resource_app_id = azuread_application.api.client_id

    resource_access {
      id   = azuread_application.api.oauth2_permission_scope_ids["access_as_user"]
      type = "Scope"
    }
  }
}

resource "azuread_service_principal" "spa" {
  client_id = azuread_application.spa.client_id
}

resource "azuread_service_principal_delegated_permission_grant" "spa_to_api" {
  service_principal_object_id          = azuread_service_principal.spa.object_id
  resource_service_principal_object_id = azuread_service_principal.api.object_id
  claim_values                         = ["access_as_user"]
}
