variable "app_short_name" {
  type        = string
  description = "Company short name (this is company-level shared infrastructure, not product-level — hence 'kkb' for company kkbae, not 'mb' for the money-bae product)"
  default     = "kkb"
}

variable "location" {
  type        = string
  description = "Azure region for the resource group holding the CIAM tenant resource"
  default     = "centralus"
}

variable "location_abrv" {
  type        = string
  description = "Abbreviated location name (eus, cus, wus)"
  default     = "cus"
}

variable "ciam_data_location" {
  type        = string
  description = "CIAM tenant data residency location: one of 'United States', 'Europe', 'Asia Pacific', 'Australia'"
  default     = "United States"
}

variable "ciam_country_code" {
  type        = string
  description = "Country code for the CIAM tenant (e.g. US)"
  default     = "US"
}

variable "ciam_tenant_name" {
  type        = string
  description = "Globally-unique tenant name: alphanumeric only, max 26 chars. Becomes <name>.onmicrosoft.com and <name>.ciamlogin.com"
  default     = "kkbaeexternalid"
}

variable "ciam_display_name" {
  type        = string
  description = "Display name shown for the CIAM tenant"
  default     = "KKBAE"
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}
