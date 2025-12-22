variable "environment" {
  type        = string
  description = "Environment name (dev, test, prod)"
}

variable "location" {
  type        = string
  description = "Azure region for resources"
}

variable "location_abrv" {
  type        = string
  description = "Abbreviated location name (eus, cus, wus)"
}

variable "app_short_name" {
  type        = string
  description = "Application short name"
  default     = "mb"
}

variable "component" {
  type        = string
  description = "Component name"
  default     = "core"
}

variable "db_admin_login" {
  type        = string
  description = "PostgreSQL administrator login"
  sensitive   = true
}

variable "db_admin_password" {
  type        = string
  description = "PostgreSQL administrator password"
  sensitive   = true
}

variable "db_allow_public_access" {
  type        = bool
  description = "Allow public access to PostgreSQL (dev only)"
  default     = false
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}
