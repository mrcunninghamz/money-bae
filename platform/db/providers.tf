terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=4.1.0"
    }
  }

  backend "azurerm" {}
}

provider "azurerm" {
  subscription_id = "c6f1212c-ec19-425f-96a0-41f2db717ea8"
  features {}
  resource_provider_registrations = "none"
}
