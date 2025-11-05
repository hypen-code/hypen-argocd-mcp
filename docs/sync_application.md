# Sync Application

## Overview

The `sync_application` tool syncs an ArgoCD application to its target state in Git. This operation deploys or updates the application resources to match what's defined in the Git repository. This is one of the most commonly used operations in ArgoCD.

## Endpoint

**POST** `/api/v1/applications/{name}/sync`

## Features

- Sync application to match Git repository state
- Dry-run mode to preview changes before applying
- Selective resource sync (sync only specific resources)
- Force sync to override conflicts
- Prune orphaned resources not defined in Git
- Custom sync options (e.g., skip validation, create namespace)
- Retry strategy with configurable backoff
- Support for application namespace and project filtering
- Blocked in read-only mode for safety
- Comprehensive error handling and validation

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `application_name` | string | Yes | Name of the application to sync |
| `revision` | string | No | Specific revision to sync to (defaults to target revision in app spec) |
| `dry_run` | boolean | No | If true, preview the sync without actually performing it (default: false) |
| `prune` | boolean | No | Whether to prune resources that are no longer defined in Git (default: false) |
| `force` | boolean | No | Use force apply to override any conflicts (default: false) |
| `resources` | array | No | Specific resources to sync (if not specified, syncs all resources) |
| `sync_options` | array of strings | No | Sync options (e.g., ["Validate=false", "CreateNamespace=true"]) |
| `retry` | object | No | Retry configuration (limit, backoff_duration, backoff_max_duration, backoff_factor) |
| `app_namespace` | string | No | Application namespace for filtering |
| `project` | string | No | Project identifier for filtering |

### Resource Specification

When specifying individual resources to sync:

```json
{
  "group": "apps",           // Optional: resource group
  "kind": "Deployment",      // Required: resource kind
  "name": "my-deployment",   // Required: resource name
  "namespace": "default"     // Optional: resource namespace
}
```

### Retry Configuration

```json
{
  "limit": 5,                        // Maximum number of retry attempts
  "backoff_duration": "5s",          // Initial backoff duration
  "backoff_max_duration": "3m",      // Maximum backoff duration
  "backoff_factor": 2                // Backoff multiplier
}
```

### Common Sync Options

- `Validate=false` - Skip kubectl validation
- `CreateNamespace=true` - Create namespace if it doesn't exist
- `PruneLast=true` - Prune resources after all other resources are synced
- `ApplyOutOfSyncOnly=true` - Only apply out-of-sync resources
- `RespectIgnoreDifferences=true` - Respect ignore differences configuration
- `Replace=true` - Use replace instead of apply
- `ServerSideApply=true` - Use server-side apply
- `SkipDryRunOnMissingResource=true` - Skip dry-run for missing resources

## Response

Returns an optimized summary containing:

```json
{
  "name": "guestbook",
  "dry_run": false,
  "sync_status": "Synced",
  "sync_revision": "abc123def456",
  "health_status": "Progressing",
  "target_revision": "HEAD",
  "prune_enabled": false,
  "force_enabled": false,
  "sync_options": [],
  "resources_count": null
}
```

### Response Fields

- `name`: Application name
- `dry_run`: Whether this was a dry-run operation
- `sync_status`: Current sync status after sync
- `sync_revision`: Current sync revision after sync
- `health_status`: Current health status after sync
- `target_revision`: Target revision that was synced to
- `prune_enabled`: Whether pruning was enabled
- `force_enabled`: Whether force was enabled
- `sync_options`: Sync options that were applied
- `resources_count`: Number of resources synced (null if all resources were synced)

## Usage Examples

### Basic Sync

Sync entire application to target revision:

```json
{
  "application_name": "guestbook"
}
```

### Dry Run Mode

Preview sync without actually performing it:

```json
{
  "application_name": "guestbook",
  "dry_run": true
}
```

### Sync to Specific Revision

Sync to a specific Git commit/tag:

```json
{
  "application_name": "guestbook",
  "revision": "v1.2.3"
}
```

### Sync with Prune

Sync and remove resources not in Git:

```json
{
  "application_name": "guestbook",
  "prune": true
}
```

### Force Sync

Force sync to override conflicts:

```json
{
  "application_name": "guestbook",
  "force": true
}
```

### Selective Resource Sync

Sync only specific resources:

```json
{
  "application_name": "guestbook",
  "resources": [
    {
      "group": "apps",
      "kind": "Deployment",
      "name": "guestbook-ui",
      "namespace": "default"
    },
    {
      "kind": "Service",
      "name": "guestbook-ui",
      "namespace": "default"
    }
  ]
}
```

### Sync with Options

Use custom sync options:

```json
{
  "application_name": "guestbook",
  "sync_options": [
    "Validate=false",
    "CreateNamespace=true",
    "ServerSideApply=true"
  ]
}
```

### Sync with Retry

Configure retry behavior:

```json
{
  "application_name": "guestbook",
  "retry": {
    "limit": 5,
    "backoff_duration": "5s",
    "backoff_max_duration": "3m",
    "backoff_factor": 2
  }
}
```

### Complete Example with All Options

```json
{
  "application_name": "my-app",
  "revision": "v1.0.0",
  "dry_run": true,
  "prune": true,
  "force": true,
  "resources": [
    {
      "group": "apps",
      "kind": "Deployment",
      "name": "my-deployment",
      "namespace": "default"
    }
  ],
  "sync_options": [
    "Validate=false",
    "CreateNamespace=true"
  ],
  "retry": {
    "limit": 3,
    "backoff_duration": "5s"
  },
  "app_namespace": "argocd",
  "project": "production"
}
```

## Output Format

The tool returns a formatted text output followed by JSON data:

```
Sync Completed for application 'guestbook'

Target Revision: HEAD
Current Sync Revision: abc123def456

Status:
  Sync Status: Synced
  Health Status: Progressing

Configuration:
  Dry Run: false
  Prune Enabled: false
  Force Enabled: false
  Resources Synced: all

✅ Sync completed successfully.
    Monitor the application to ensure it reaches the desired state.

--- JSON Data ---
{
  "name": "guestbook",
  "dry_run": false,
  ...
}
```

### Dry Run Output

When using dry-run mode:

```
Sync (Dry Run) for application 'guestbook'

Target Revision: HEAD

Status:
  Sync Status: OutOfSync
  Health Status: Healthy

Configuration:
  Dry Run: true
  Prune Enabled: false
  Force Enabled: false
  Resources Synced: all

⚠️  Note: This was a dry run. No actual changes were made.
    Run without dry_run=true to perform the actual sync.
```

### Partial Sync Output

When syncing specific resources:

```
Sync Completed for application 'guestbook'

...

Configuration:
  Dry Run: false
  Prune Enabled: false
  Force Enabled: false
  Resources Synced: 2 (partial sync)
  Sync Options: Validate=false, CreateNamespace=true
```

## Error Handling

The tool handles various error scenarios:

### Application Not Found (404)

```
ArgoCD API error (404): Application 'nonexistent-app' not found
```

### Authentication Error (401)

```
ArgoCD API error (401): Invalid authentication token
```

### Permission Denied (403)

```
ArgoCD API error (403): User does not have permission to sync application
```

### Sync Conflict (409)

```
ArgoCD API error (409): Application is already being synced
```

### Read-Only Mode

If the MCP server is in read-only mode:

```
Cannot sync application in read-only mode. This operation modifies application state.
```

## Best Practices

1. **Always use dry-run first**: Preview the sync with `dry_run: true` before executing, especially in production
2. **Monitor after sync**: Watch the application to ensure it reaches the desired state (sync status and health status)
3. **Be cautious with force**: Only use force when you understand the implications and know it's necessary
4. **Be careful with prune**: Pruning removes resources - ensure you know what will be deleted
5. **Use selective sync for large apps**: Sync specific resources when you only need to update certain components
6. **Configure retries appropriately**: Use retry configuration for unreliable environments
7. **Check sync windows**: Verify the application isn't in a blocked sync window before syncing
8. **Use meaningful revisions**: When syncing to specific revisions, use tags or meaningful commits
9. **Review sync options**: Understand what each sync option does before using it
10. **Document sync operations**: Keep track of manual syncs and their reasons

## Common Use Cases

### Deploy New Version

```json
{
  "application_name": "my-app",
  "revision": "v2.0.0",
  "prune": true
}
```

### Emergency Rollforward

```json
{
  "application_name": "my-app",
  "revision": "hotfix-branch",
  "force": true
}
```

### Clean Up Orphaned Resources

```json
{
  "application_name": "my-app",
  "prune": true
}
```

### Fix Configuration Drift

```json
{
  "application_name": "my-app",
  "force": true,
  "prune": true
}
```

### Test Changes Safely

```json
{
  "application_name": "my-app",
  "dry_run": true,
  "revision": "feature-branch"
}
```

### Update Single Service

```json
{
  "application_name": "microservices-app",
  "resources": [
    {
      "group": "apps",
      "kind": "Deployment",
      "name": "user-service"
    }
  ]
}
```

## Read-Only Mode

The `sync_application` tool is a write operation and is **blocked in read-only mode**. You will receive an error if you try to use it when `ARGOCD_READ_ONLY=true`.

To perform syncs, ensure the MCP server is not in read-only mode.

## Sync vs Rollback

| Feature | Sync | Rollback |
|---------|------|----------|
| Direction | Forward to Git state | Backward to previous deployment |
| Source | Git repository | ArgoCD history |
| Use Case | Deploy new changes | Undo recent changes |
| Revision | Any Git revision | Must be in history |
| Typical Scenario | Normal deployment | Emergency recovery |

## Implementation Details

### Models

The sync functionality uses the following models:

- `ApplicationSyncSummary`: Optimized output summary
- `SyncResource`: Resource specification for partial sync
- `SyncStrategy`: Strategy configuration (force apply/hook)
- `RetryStrategy`: Retry configuration
- `Backoff`: Backoff strategy configuration

### API Client Method

```rust
pub async fn sync_application(
    &self,
    name: String,
    revision: Option<String>,
    dry_run: Option<bool>,
    prune: Option<bool>,
    force: Option<bool>,
    resources: Option<Vec<SyncResource>>,
    sync_options: Option<Vec<String>>,
    retry: Option<RetryStrategy>,
    app_namespace: Option<String>,
    project: Option<String>,
) -> Result<ApplicationSyncSummary>
```

### Test Coverage

The sync functionality includes comprehensive tests covering:

- Basic sync operation
- Dry-run mode
- Prune functionality
- Force sync
- Specific revision sync
- Selective resource sync
- Sync options
- Retry configuration
- Application namespace and project filtering
- All options combined
- Error cases (404, 401, 403, 500)
- Network timeout scenarios
- Malformed responses

All 17 tests pass successfully.

## Related Tools

- `get_application`: Check current application state before syncing
- `server_side_diff`: Preview what will change during sync
- `rollback_application`: Undo a sync by rolling back to previous version
- `list_resource_events`: View events after sync to troubleshoot issues
- `pod_logs`: Monitor logs after sync to verify deployment
- `resource_tree`: Check resource status after sync

## References

- [ArgoCD CLI Sync Command](https://argo-cd.readthedocs.io/en/latest/user-guide/commands/argocd_app_sync/)
- [ArgoCD API Documentation](https://cd.apps.argoproj.io/swagger-ui)
- [ArgoCD Sync Options](https://argo-cd.readthedocs.io/en/stable/user-guide/sync-options/)
- [ArgoCD Sync Phases and Waves](https://argo-cd.readthedocs.io/en/stable/user-guide/sync-waves/)
