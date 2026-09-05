output "client_id" {
  value = azuread_application.api.client_id
}

output "identifier_uri" {
  value = one(azuread_application.api.identifier_uris)
}

output "scope_id" {
  value = random_uuid.access_as_user.result
}

output "service_principal_object_id" {
  value = azuread_service_principal.api.object_id
}
