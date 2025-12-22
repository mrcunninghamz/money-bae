output "server_id" {
  value       = azurerm_postgresql_flexible_server.main.id
  description = "ID of the PostgreSQL Flexible Server"
}

output "server_name" {
  value       = azurerm_postgresql_flexible_server.main.name
  description = "Name of the PostgreSQL Flexible Server"
}

output "server_fqdn" {
  value       = azurerm_postgresql_flexible_server.main.fqdn
  description = "Fully qualified domain name of the PostgreSQL server"
}

output "database_name" {
  value       = azurerm_postgresql_flexible_server_database.main.name
  description = "Name of the PostgreSQL database"
}

output "connection_string" {
  value       = "postgresql://${var.administrator_login}@${azurerm_postgresql_flexible_server.main.name}:${var.administrator_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/${azurerm_postgresql_flexible_server_database.main.name}?sslmode=require"
  description = "PostgreSQL connection string"
  sensitive   = true
}
