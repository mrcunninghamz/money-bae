variable "ciam_tenant_id" {
  type        = string
  description = "Tenant ID of the Entra External ID (CIAM) tenant — pass with -var, sourced from `terraform -chdir=../tenant output -raw tenant_id`. Not stored in tfvars since it's an output of a different Terraform state, not a static config value."
}

variable "local_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the local environment"
  # Trailing slash required: Entra rejects a redirect URI with no path
  # segment (e.g. "http://localhost:3000") unless it ends in "/".
  default = "http://localhost:3000/"
}

variable "dev_redirect_uri" {
  type        = string
  description = "SPA redirect URI for the dev environment: the web client's deployed CloudFront domain (MoneyBaeWebClient-Dev stack's DistributionDomainName output). No custom domain exists yet, so this is pinned to the actual distribution's domain rather than computed — needs a manual update if the distribution is ever replaced."
  # Trailing slash required: Entra rejects a redirect URI with no path
  # segment unless it ends in "/".
  default = "https://d91s2th9i95hi.cloudfront.net/"
}
