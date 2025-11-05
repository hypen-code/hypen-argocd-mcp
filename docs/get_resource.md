# Get Resource

## Overview

The `get_resource` tool retrieves a specific Kubernetes resource from an ArgoCD application. It returns detailed resource manifest including metadata, spec, and status information.

## Tool Name
`get_resource`

## Description
Get a specific Kubernetes resource from an ArgoCD application. Returns detailed resource manifest including metadata, spec, and status. Use this to inspect the current state of individual resources like Pods, Services, Deployments, ConfigMaps, etc. The manifest is parsed to extract key fields like API version, kind, name, namespace, labels, and status summary.

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `application_name` | string | The name of the ArgoCD application |
| `resource_name` | string | The name of the specific resource to retrieve |
| `version` | string | The Kubernetes API version (e.g., "v1", "apps/v1") |
| `kind` | string | The resource kind (e.g., "Pod", "Service", "Deployment") |

### Optional Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `namespace` | string | The namespace of the resource (optional for cluster-scoped resources) |
| `group` | string | The API group (empty for core resources, "apps" for deployments, etc.) |
| `app_namespace` | string | The namespace of the ArgoCD application |
| `project` | string | The ArgoCD project identifier |

## Response

The tool returns an optimized summary with:
- **Resource identification**: Application name, resource name, kind, version, group
- **Namespace**: The Kubernetes namespace containing the resource
- **Manifest Summary**: Parsed metadata including:
  - API version and kind
  - Name and namespace
  - Labels (up to 5 shown, with count if more)
  - Annotations count
  - Creation timestamp
  - Status summary (parsed from status field if available)
- **Full Manifest**: Complete YAML manifest (first 50 lines shown, with indication if truncated)
- **JSON Data**: Structured JSON representation for programmatic consumption

## Use Cases

1. **Inspect Pod Status**: Check the current state of a pod including container status, restart count, and events
2. **Review Service Configuration**: Examine service endpoints, ports, and selectors
3. **Verify Deployment State**: Check replica counts, container images, and deployment status
4. **Inspect ConfigMaps/Secrets**: Review configuration data and keys
5. **Troubleshooting**: Get detailed resource information for debugging issues
6. **Audit Resource Configuration**: Review labels, annotations, and resource specifications

## Examples

### Example 1: Get a Pod

```json
{
  "application_name": "my-app",
  "namespace": "production",
  "resource_name": "web-server-abc123",
  "version": "v1",
  "kind": "Pod"
}
```

### Example 2: Get a Deployment

```json
{
  "application_name": "backend-service",
  "namespace": "production",
  "resource_name": "api-deployment",
  "version": "v1",
  "group": "apps",
  "kind": "Deployment"
}
```

### Example 3: Get a Service

```json
{
  "application_name": "frontend",
  "namespace": "staging",
  "resource_name": "web-service",
  "version": "v1",
  "kind": "Service"
}
```

### Example 4: Get a ConfigMap

```json
{
  "application_name": "config-manager",
  "namespace": "default",
  "resource_name": "app-config",
  "version": "v1",
  "kind": "ConfigMap"
}
```

## Response Example

```
Resource: nginx-deployment (Deployment)
Application: production-app
Version: v1
Group: apps
Namespace: production

Manifest Summary:
  API Version: apps/v1
  Kind: Deployment
  Name: nginx-deployment
  Namespace: production
  Labels (3):
    app: nginx
    env: production
    version: 1.0
  Annotations Count: 2
  Created: 2025-01-01T00:00:00Z
  Status: 3/3 replicas ready

📄 Full Manifest:
────────────────────────────────────────────────────────────────────────────────
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  namespace: production
  labels:
    app: nginx
    env: production
    version: "1.0"
...
```

## Error Handling

The tool will return an error in the following cases:

- **Client not initialized**: ArgoCD client is not properly configured
- **Resource not found**: The specified resource doesn't exist in the application
- **Invalid parameters**: Missing required parameters or invalid values
- **API errors**: ArgoCD API returns an error (e.g., unauthorized, forbidden, server error)
- **Network errors**: Unable to connect to ArgoCD server

## Best Practices

1. **Resource Identification**: Ensure you have the correct resource name, kind, and version
2. **Namespace Specification**: Always specify the namespace for namespaced resources
3. **API Group**: Specify the group for non-core resources (e.g., "apps" for Deployments)
4. **Use with resource_tree**: First use `resource_tree` to discover available resources, then use `get_resource` for details
5. **Status Monitoring**: Use this tool to monitor resource status and troubleshoot issues

## Related Tools

- `resource_tree`: List all resources in an application to discover resource names
- `patch_resource`: Modify a resource after inspecting it with get_resource
- `pod_logs`: Get logs from a pod after retrieving its details
- `list_resource_events`: Get events related to a resource for troubleshooting

## API Endpoint

```
GET /api/v1/applications/{name}/resource
```

## Context Optimization

This tool is optimized for LLM context efficiency by:
- Parsing and summarizing key manifest fields
- Truncating long manifests (showing first 50 lines)
- Extracting and presenting only relevant labels (up to 5)
- Providing status summaries instead of full status objects
- Offering both human-readable and JSON formats

## Read-Only Tool

This is a read-only tool that does not modify any resources. It is safe to use in production environments and is available in read-only mode.
