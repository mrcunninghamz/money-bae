output "resource_group_name" {
  value       = azurerm_resource_group.main.name
  description = "Name of the resource group"
}

output "resource_group_location" {
  value       = azurerm_resource_group.main.location
  description = "Location of the resource group"
}

output "postgresql_server_name" {
  value       = module.postgresql.server_name
  description = "Name of the PostgreSQL server"
}

output "postgresql_server_fqdn" {
  value       = module.postgresql.server_fqdn
  description = "FQDN of the PostgreSQL server"
}

output "postgresql_database_name" {
  value       = module.postgresql.database_name
  description = "Name of the PostgreSQL database"
}

output "postgresql_connection_string" {
  value       = module.postgresql.connection_string
  description = "PostgreSQL connection string"
  sensitive   = true
}
