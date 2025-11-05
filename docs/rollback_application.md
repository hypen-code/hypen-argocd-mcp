# Rollback Application

## Overview

The `rollback_application` tool allows you to rollback an ArgoCD application to a previous deployed version by History ID. This operation reverts the application to a specific point in its deployment history.

## Endpoint

**POST** `/api/v1/applications/{name}/rollback`

## Features

- Rollback to any previous deployed version by History ID
- Dry-run mode to preview changes before applying
- Optional pruning of resources that no longer exist in the target revision
- Support for application namespace and project filtering
- Blocked in read-only mode for safety
- Comprehensive error handling and validation

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `application_name` | string | Yes | Name of the application to rollback |
| `id` | integer | Yes | History ID to rollback to. Use 0 to rollback to the previous version |
| `dry_run` | boolean | No | If true, preview the rollback without actually performing it (default: false) |
| `prune` | boolean | No | Whether to prune resources that are no longer defined in the target revision (default: false) |
| `app_namespace` | string | No | Application namespace for filtering |
| `project` | string | No | Project identifier for filtering |

## Response

Returns an optimized summary containing:

```json
{
  "name": "guestbook",
  "rolled_back_to_id": 5,
  "dry_run": false,
  "sync_status": "Synced",
  "sync_revision": "abc123def456",
  "health_status": "Healthy",
  "target_revision": "abc123",
  "prune_enabled": false
}
```

### Response Fields

- `name`: Application name
- `rolled_back_to_id`: The History ID that was rolled back to
- `dry_run`: Whether this was a dry-run operation
- `sync_status`: Current sync status after rollback
- `sync_revision`: Current sync revision after rollback
- `health_status`: Current health status after rollback
- `target_revision`: Target revision after rollback
- `prune_enabled`: Whether pruning was enabled

## Usage Examples

### Basic Rollback

Rollback to a specific history ID:

```json
{
  "application_name": "guestbook",
  "id": 5
}
```

### Rollback to Previous Version

Use ID 0 to rollback to the immediately previous version:

```json
{
  "application_name": "guestbook",
  "id": 0
}
```

### Dry Run Mode

Preview the rollback without actually performing it:

```json
{
  "application_name": "guestbook",
  "id": 3,
  "dry_run": true
}
```

### Rollback with Prune

Rollback and prune resources that no longer exist in the target revision:

```json
{
  "application_name": "guestbook",
  "id": 5,
  "prune": true
}
```

### With Namespace and Project

Rollback an application in a specific namespace and project:

```json
{
  "application_name": "backend-api",
  "id": 10,
  "app_namespace": "argocd",
  "project": "production"
}
```

### Complete Example with All Options

```json
{
  "application_name": "my-app",
  "id": 7,
  "dry_run": true,
  "prune": true,
  "app_namespace": "argocd",
  "project": "default"
}
```

## Output Format

The tool returns a formatted text output followed by JSON data:

```
Rollback Completed for application 'guestbook'

Rolled back to History ID: 5
Target Revision: abc123
Current Sync Revision: abc123def456

Status:
  Sync Status: Synced
  Health Status: Healthy

Options:
  Dry Run: false
  Prune Enabled: false

✅ Rollback completed successfully.
    Monitor the application to ensure it reaches the desired state.

--- JSON Data ---
{
  "name": "guestbook",
  "rolled_back_to_id": 5,
  ...
}
```

### Dry Run Output

When using dry-run mode:

```
Rollback (Dry Run) for application 'guestbook'

...

⚠️  Note: This was a dry run. No actual changes were made.
    Run without dry_run=true to perform the actual rollback.
```

## Error Handling

The tool handles various error scenarios:

### Application Not Found (404)

```
ArgoCD API error (404): Application 'nonexistent-app' not found
```

### Invalid History ID (400)

```
ArgoCD API error (400): History ID 999 not found for application 'test-app'
```

### Authentication Error (401)

```
ArgoCD API error (401): Invalid authentication token
```

### Permission Denied (403)

```
ArgoCD API error (403): User does not have permission to rollback application
```

### Read-Only Mode

If the MCP server is in read-only mode:

```
Cannot rollback application in read-only mode. This operation modifies application state.
```

## Best Practices

1. **Always use dry-run first**: Preview the rollback with `dry_run: true` before executing
2. **Monitor after rollback**: Watch the application to ensure it reaches the desired state
3. **Check history**: Use `revision_metadata` or ArgoCD UI to verify the correct history ID
4. **Be cautious with prune**: Only enable pruning if you're sure you want to remove resources
5. **Document rollbacks**: Keep track of why and when rollbacks are performed

## Read-Only Mode

The `rollback_application` tool is a write operation and is **blocked in read-only mode**. If you try to use it when `ARGOCD_READ_ONLY=true`, you will receive an error:

```
Cannot rollback application in read-only mode. This operation modifies application state.
```

To perform rollbacks, ensure the MCP server is not in read-only mode.

## Implementation Details

### Models

The rollback functionality uses the following models:

- `ApplicationRollbackSummary`: Optimized output summary
- `ApplicationRollbackRequest`: Request structure (internal)
- `ApplicationRollbackResponse`: Response structure (internal)

### API Client Method

```rust
pub async fn rollback_application(
    &self,
    name: String,
    id: i64,
    dry_run: Option<bool>,
    prune: Option<bool>,
    app_namespace: Option<String>,
    project: Option<String>,
) -> Result<ApplicationRollbackSummary>
```

### Test Coverage

The rollback functionality includes comprehensive tests covering:

- Basic rollback operation
- Dry-run mode
- Prune functionality
- Application namespace and project filtering
- All options combined
- Error cases (404, 400, 401, 403, 500)
- Special characters in application names
- Network timeout scenarios
- Malformed responses

All 15 tests pass successfully.

## Related Tools

- `get_application`: Get current application state
- `revision_metadata`: Get metadata about a specific revision
- `list_resource_events`: View events related to the rollback
- `pod_logs`: Monitor logs after rollback

## References

- [ArgoCD CLI Rollback Command](https://argo-cd.readthedocs.io/en/latest/user-guide/commands/argocd_app_rollback/)
- [ArgoCD API Documentation](https://cd.apps.argoproj.io/swagger-ui)
- [ArgoCD Application History](https://argo-cd.readthedocs.io/en/stable/user-guide/history_and_rollback/)
