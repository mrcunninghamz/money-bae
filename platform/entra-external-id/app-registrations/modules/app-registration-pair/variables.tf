variable "env_name" {
  type        = string
  description = "Environment name, used in app registration display names (local, dev)"
}

variable "redirect_uri" {
  type        = string
  description = "SPA redirect URI for this environment (e.g. http://localhost:3000)"
}
