terraform {
  required_version = ">= 1.1"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=4.1.0"
    }
  }

  backend "azurerm" {}
}

provider "azurerm" {
  subscription_id = "085f952f-488d-4c4d-bd33-0fcf8fd37e17"
  features {}
  resource_provider_registrations = "none"
}
