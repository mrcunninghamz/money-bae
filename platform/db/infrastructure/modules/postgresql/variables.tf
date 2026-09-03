variable "server_name" {
  type        = string
  description = "Name of the PostgreSQL Flexible Server"
}

variable "resource_group_name" {
  type        = string
  description = "Name of the resource group"
}

variable "location" {
  type        = string
  description = "Azure region for the server"
}

variable "database_name" {
  type        = string
  description = "Name of the database to create"
  default     = "money_bae"
}

variable "administrator_login" {
  type        = string
  description = "Administrator login for PostgreSQL server"
  sensitive   = true
}

variable "administrator_password" {
  type        = string
  description = "Administrator password for PostgreSQL server"
  sensitive   = true
}

variable "allow_public_access" {
  type        = bool
  description = "Allow public access to the database (dev only)"
  default     = false
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}
