# Patch Resource

## Overview

The `patch_resource` tool modifies a specific Kubernetes resource in an ArgoCD application using various patch strategies. This is a write operation that directly updates resources managed by ArgoCD.

## Tool Name
`patch_resource`

## Description
Patch a Kubernetes resource in an ArgoCD application. This operation modifies a specific resource using JSON patch, merge patch, or strategic merge patch formats. Returns the updated resource manifest. Common use cases include scaling deployments, updating environment variables, modifying labels/annotations, and changing resource configurations.

⚠️ **NOTE**: This is a write operation and is blocked in read-only mode.

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `application_name` | string | The name of the ArgoCD application |
| `resource_name` | string | The name of the resource to patch |
| `version` | string | The Kubernetes API version (e.g., "v1", "apps/v1") |
| `kind` | string | The resource kind (e.g., "Deployment", "Service", "ConfigMap") |
| `patch` | string | The patch content as a JSON string |

### Optional Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `namespace` | string | The namespace of the resource |
| `group` | string | The API group (empty for core resources) |
| `patch_type` | string | Patch strategy type (see Patch Types below) |
| `app_namespace` | string | The namespace of the ArgoCD application |
| `project` | string | The ArgoCD project identifier |

## Patch Types

| Patch Type | Description | Use Case |
|------------|-------------|----------|
| `application/json-patch+json` | RFC 6902 JSON Patch | Precise operations (add, remove, replace, move, copy, test) |
| `application/merge-patch+json` | RFC 7396 Merge Patch | Simple merging of JSON objects |
| `application/strategic-merge-patch+json` | Kubernetes Strategic Merge | Kubernetes-aware merging with list merge strategies |

**Default**: If not specified, ArgoCD will use strategic merge patch for most resources.

## Response

The tool returns an optimized summary with:
- **Patch Confirmation**: Success indicator and resource identification
- **Updated Resource Details**: Application name, resource name, kind, version, group
- **Namespace**: The Kubernetes namespace
- **Updated Manifest Summary**: Parsed metadata of the patched resource including:
  - API version and kind
  - Name and namespace
  - Updated labels
  - Annotations count
  - Status summary
- **Updated Manifest**: Complete YAML manifest showing the patched state (first 50 lines)
- **JSON Data**: Structured JSON representation for programmatic consumption

## Use Cases

### 1. Scale Deployment
Scale the number of replicas in a deployment.

```json
{
  "application_name": "backend-app",
  "namespace": "production",
  "resource_name": "api-deployment",
  "version": "v1",
  "group": "apps",
  "kind": "Deployment",
  "patch": "{\"spec\": {\"replicas\": 5}}",
  "patch_type": "application/merge-patch+json"
}
```

### 2. Update Container Image
Update the container image version in a deployment.

```json
{
  "application_name": "web-app",
  "namespace": "production",
  "resource_name": "frontend-deployment",
  "version": "v1",
  "group": "apps",
  "kind": "Deployment",
  "patch": "{\"spec\": {\"template\": {\"spec\": {\"containers\": [{\"name\": \"web\", \"image\": \"nginx:1.22-alpine\"}]}}}}",
  "patch_type": "application/strategic-merge-patch+json"
}
```

### 3. Add Label to Resource
Add or update a label on any resource.

```json
{
  "application_name": "my-app",
  "namespace": "default",
  "resource_name": "my-service",
  "version": "v1",
  "kind": "Service",
  "patch": "{\"metadata\": {\"labels\": {\"environment\": \"production\"}}}",
  "patch_type": "application/strategic-merge-patch+json"
}
```

### 4. Update Environment Variables
Update environment variables in a deployment's container.

```json
{
  "application_name": "api-service",
  "namespace": "production",
  "resource_name": "api-deployment",
  "version": "v1",
  "group": "apps",
  "kind": "Deployment",
  "patch": "{\"spec\": {\"template\": {\"spec\": {\"containers\": [{\"name\": \"api\", \"env\": [{\"name\": \"LOG_LEVEL\", \"value\": \"debug\"}]}]}}}}",
  "patch_type": "application/strategic-merge-patch+json"
}
```

### 5. Update ConfigMap Data
Modify configuration data in a ConfigMap.

```json
{
  "application_name": "config-app",
  "namespace": "default",
  "resource_name": "app-config",
  "version": "v1",
  "kind": "ConfigMap",
  "patch": "{\"data\": {\"app.properties\": \"key=new-value\\nenabled=true\"}}",
  "patch_type": "application/merge-patch+json"
}
```

### 6. JSON Patch Operations
Use RFC 6902 JSON Patch for precise operations.

```json
{
  "application_name": "app",
  "namespace": "default",
  "resource_name": "my-config",
  "version": "v1",
  "kind": "ConfigMap",
  "patch": "[{\"op\": \"replace\", \"path\": \"/data/config.json\", \"value\": \"{\\\"setting\\\": \\\"value\\\"}\"}]",
  "patch_type": "application/json-patch+json"
}
```

## Response Example

```
✅ Patched Resource: api-deployment (Deployment)
Application: production-app
Version: v1
Group: apps
Namespace: production

Updated Manifest Summary:
  API Version: apps/v1
  Kind: Deployment
  Name: api-deployment
  Namespace: production
  Labels (4):
    app: api
    env: production
    version: 2.0
    patched: true
  Annotations Count: 3
  Status: 5/5 replicas ready

📄 Updated Manifest:
────────────────────────────────────────────────────────────────────────────────
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-deployment
  namespace: production
  labels:
    app: api
    env: production
    version: "2.0"
    patched: "true"
spec:
  replicas: 5
...

💡 Tip: Monitor the resource to ensure it reaches the desired state.
```

## Error Handling

The tool will return an error in the following cases:

- **Client not initialized**: ArgoCD client is not properly configured
- **Read-only mode**: Attempting to patch in read-only mode
- **Resource not found**: The specified resource doesn't exist
- **Invalid patch**: Malformed patch document or incompatible patch type
- **Validation errors**: Patch violates Kubernetes resource validation rules
- **API errors**: ArgoCD API returns an error (unauthorized, forbidden, conflict)
- **Network errors**: Unable to connect to ArgoCD server

## Best Practices

1. **Test with dry-run**: Always test patches in a non-production environment first
2. **Use appropriate patch type**:
   - JSON Patch for precise operations
   - Merge Patch for simple updates
   - Strategic Merge Patch for Kubernetes resources
3. **Specify namespace**: Always specify namespace for namespaced resources
4. **Validate patch format**: Ensure patch is valid JSON
5. **Monitor after patching**: Use `get_resource` to verify the patch was applied correctly
6. **Consider ArgoCD sync**: Be aware that ArgoCD may revert changes if they conflict with Git
7. **Use with sync operations**: For permanent changes, update the Git repository and use `sync_application`

## Patch Strategy Guide

### Strategic Merge Patch (Recommended for Kubernetes)
- **Advantages**: Kubernetes-aware, handles lists intelligently
- **Use for**: Most Kubernetes resource updates
- **Example**: Updating container images, environment variables, replicas

### Merge Patch
- **Advantages**: Simple, intuitive for object merging
- **Use for**: Simple updates to ConfigMaps, Services
- **Limitation**: Cannot remove fields (sets to null instead)

### JSON Patch
- **Advantages**: Precise control, can perform any operation
- **Use for**: Complex updates, removing specific fields, testing values
- **Limitation**: Requires exact path specification

## Important Notes

1. **ArgoCD Sync Behavior**: Changes made via `patch_resource` may be overwritten by ArgoCD if the application is synced and the change conflicts with the Git repository.

2. **Read-Only Mode**: This operation is disabled when `ARGOCD_READ_ONLY=true` environment variable is set.

3. **Drift Detection**: After patching, ArgoCD will detect drift between Git and the cluster if the patch changes differ from Git.

4. **Consider Alternatives**: For permanent changes, consider:
   - Updating the Git repository
   - Using `sync_application` after Git updates
   - Configuring proper sync policies

## Related Tools

- `get_resource`: Inspect a resource before patching
- `sync_application`: Sync application from Git after patching
- `resource_tree`: List all resources to find patchable resources
- `get_manifests`: Review what's defined in Git vs. what's been patched

## API Endpoint

```
POST /api/v1/applications/{name}/resource
```

## Context Optimization

This tool is optimized for LLM context efficiency by:
- Returning summarized manifest information
- Truncating long manifests
- Extracting key fields for display
- Providing both human-readable and JSON formats

## Security Considerations

- **Authorization**: Requires appropriate RBAC permissions in ArgoCD
- **Audit**: All patch operations are logged by ArgoCD
- **Validation**: Kubernetes validates all patches before applying
- **Read-Only Mode**: Can be disabled globally with `ARGOCD_READ_ONLY=true`
