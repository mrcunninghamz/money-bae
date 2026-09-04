# Postman collections

`money-bae-api.postman_collection.json` — import this collection.
`money-bae-api-local.postman_environment.json` / `money-bae-api-dev.postman_environment.json` — import both environments, select one at a time.

## OAuth setup (one-time, per environment)

After `platform/entra-external-id/app-registrations` has been applied,
fill in each environment's `clientId`/`authUrl`/`tokenUrl`/`scope`
variables:

- `clientId` — `terraform -chdir=platform/entra-external-id/app-registrations output -raw spa_client_id_local` (or `_dev`)
- `authUrl` — `https://kkbaeexternalid.ciamlogin.com/kkbaeexternalid.onmicrosoft.com/oauth2/v2.0/authorize`
- `tokenUrl` — `https://kkbaeexternalid.ciamlogin.com/kkbaeexternalid.onmicrosoft.com/oauth2/v2.0/token`
- `scope` — `terraform -chdir=platform/entra-external-id/app-registrations output -raw api_identifier_uri_local` (or `_dev`) with `/access_as_user` appended

Then in Postman, on a request using this collection's auth, click
"Get New Access Token" — it uses Authorization Code with PKCE, no
client secret, since these are SPA (public client) app registrations.
