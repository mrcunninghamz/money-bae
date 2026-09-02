resource "azurerm_resource_group" "main" {
  name     = "rg-${var.app_short_name}-${var.component}-${var.location_abrv}-${var.environment}"
  location = var.location
  tags     = var.tags
}

module "postgresql" {
  source = "./modules/postgresql"

  server_name            = "psql-${var.app_short_name}-${var.component}-${var.location_abrv}-${var.environment}"
  resource_group_name    = azurerm_resource_group.main.name
  location               = azurerm_resource_group.main.location
  database_name          = "money_bae"
  administrator_login    = var.db_admin_login
  administrator_password = var.db_admin_password
  allow_public_access    = var.db_allow_public_access
  tags                   = var.tags
}
