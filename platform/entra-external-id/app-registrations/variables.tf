variable "ciam_tenant_id" {
  type        = string
  description = "Tenant ID of the Entra External ID (CIAM) tenant — pass with -var, sourced from `terraform -chdir=../tenant output -raw tenant_id`. Not stored in tfvars since it's an output of a different Terraform state, not a static config value."
}

variable "local_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the local environment"
  default     = "http://localhost:3000"
}

variable "dev_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the dev environment: the web client's deployed CloudFront domain. Update this after platform/web-client's first `cdk deploy` — the distribution has no custom domain, so its domain is only known post-deploy."
  default     = "https://REPLACE-AFTER-FIRST-WEB-CLIENT-DEPLOY.cloudfront.net"
}
