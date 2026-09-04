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
