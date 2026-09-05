output "spa_client_id_local" {
  value = module.spa_local.client_id
}

output "spa_client_id_dev" {
  value = module.spa_dev.client_id
}

output "api_client_id_local" {
  value = module.api_local.client_id
}

output "api_client_id_dev" {
  value = module.api_dev.client_id
}

output "api_identifier_uri_local" {
  value = module.api_local.identifier_uri
}

output "api_identifier_uri_dev" {
  value = module.api_dev.identifier_uri
}
