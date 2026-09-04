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
