locals {
  # oauth.pstmn.io is Postman's OAuth2 callback proxy, added to every SPA
  # app's redirect URIs so the Postman collection (see ../CLAUDE.md) can
  # obtain real tokens for manual API testing without a second app
  # registration.
  postman_callback_redirect_uri = "https://oauth.pstmn.io/v1/callback"
}

module "api_local" {
  source = "./modules/api-registration"

  display_name = "money-bae-api-local"
}

module "api_dev" {
  source = "./modules/api-registration"

  display_name = "money-bae-api-dev"
}

module "spa_local" {
  source = "./modules/spa-registration"

  display_name                    = "money-bae-local"
  redirect_uris                   = [var.local_redirect_uri, local.postman_callback_redirect_uri]
  api_client_id                   = module.api_local.client_id
  api_scope_id                    = module.api_local.scope_id
  api_service_principal_object_id = module.api_local.service_principal_object_id
}

module "spa_dev" {
  source = "./modules/spa-registration"

  display_name                    = "money-bae-dev"
  redirect_uris                   = [var.dev_redirect_uri, local.postman_callback_redirect_uri]
  api_client_id                   = module.api_dev.client_id
  api_scope_id                    = module.api_dev.scope_id
  api_service_principal_object_id = module.api_dev.service_principal_object_id
}
