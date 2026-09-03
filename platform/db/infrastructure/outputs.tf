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

output "postgresql_database_names" {
  value       = module.postgresql.database_names
  description = "Names of the PostgreSQL databases"
}

output "postgresql_connection_strings" {
  value       = module.postgresql.connection_strings
  description = "Map of database name to PostgreSQL connection string"
  sensitive   = true
}
