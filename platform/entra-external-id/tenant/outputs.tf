output "tenant_id" {
  value       = azapi_resource.ciam_tenant.output.properties.tenantId
  description = "Tenant ID (GUID) of the new Entra External ID (CIAM) tenant"
}

output "tenant_domain" {
  value       = "${var.ciam_tenant_name}.onmicrosoft.com"
  description = "Default domain of the new CIAM tenant"
}

output "resource_group_name" {
  value = azurerm_resource_group.identity.name
}
