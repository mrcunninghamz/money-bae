terraform {
  required_version = ">= 1.1"

  required_providers {
    azuread = {
      source  = "hashicorp/azuread"
      version = "=3.9.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "=3.9.0"
    }
  }

  backend "azurerm" {}
}

provider "azuread" {
  tenant_id = var.ciam_tenant_id
}
