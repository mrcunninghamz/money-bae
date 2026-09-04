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
