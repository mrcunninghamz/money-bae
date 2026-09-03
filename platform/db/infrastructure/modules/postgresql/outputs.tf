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

output "database_names" {
  value       = [for db in azurerm_postgresql_flexible_server_database.main : db.name]
  description = "Names of the PostgreSQL databases"
}

output "connection_strings" {
  value = {
    for name, db in azurerm_postgresql_flexible_server_database.main :
    name => "postgresql://${var.administrator_login}:${var.administrator_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/${db.name}?sslmode=require"
  }
  description = "Map of database name to PostgreSQL connection string"
  sensitive   = true
}
