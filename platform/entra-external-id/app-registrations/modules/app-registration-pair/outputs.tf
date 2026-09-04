output "spa_client_id" {
  value = azuread_application.spa.client_id
}

output "api_client_id" {
  value = azuread_application.api.client_id
}

output "api_identifier_uri" {
  value = one(azuread_application.api.identifier_uris)
}
