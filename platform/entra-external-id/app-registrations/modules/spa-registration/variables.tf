variable "display_name" {
  type        = string
  description = "Display name for the SPA app registration (e.g. money-bae-local)"
}

variable "redirect_uris" {
  type        = list(string)
  description = "Redirect URIs for the SPA (PKCE) platform"
}

variable "api_client_id" {
  type        = string
  description = "Client ID of the API app registration this SPA is granted access to (an api-registration module instance's client_id output)"
}

variable "api_scope_id" {
  type        = string
  description = "Scope UUID this SPA requests (an api-registration module instance's scope_id output)"
}

variable "api_service_principal_object_id" {
  type        = string
  description = "Object ID of the API app registration's service principal, needed to grant the delegated permission (an api-registration module instance's service_principal_object_id output)"
}
