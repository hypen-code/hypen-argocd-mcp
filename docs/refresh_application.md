# Refresh Application

## Overview

The `refresh_application` tool forces ArgoCD to re-fetch application manifests from the Git repository and recompute the sync status. **This is a read-only operation that does not modify cluster state** - it only updates ArgoCD's cached view of the application. This is one of the most common troubleshooting operations in ArgoCD.

## Tool Name
`refresh_application`

## Description
Refresh an ArgoCD application from the Git repository. Forces ArgoCD to re-fetch the application manifests from Git and recompute the sync status. This is a read-only operation that does not modify cluster state - it only updates ArgoCD's cached view of the application. Use this to resolve stale sync status, update after Git changes, or troubleshoot 'stuck' applications. Returns before/after comparison showing what changed.

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `application_name` | string | The name of the ArgoCD application to refresh |

### Optional Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `refresh_type` | string | Type of refresh: "normal" or "hard" (default: "hard")<br>- "normal": Regular refresh from ArgoCD cache<br>- "hard": Force refresh from Git repository |
| `app_namespace` | string | The namespace of the ArgoCD application (if not in default ArgoCD namespace) |
| `project` | string | The ArgoCD project identifier |

## Response

The tool returns a comprehensive before/after comparison including:
- **Application name**: Name of the refreshed application
- **Refresh type**: Type of refresh performed ("normal" or "hard")
- **Repository info**: Git repository URL and target revision
- **Status comparison**:
  - **Sync status** (before/after): Shows if application is Synced, OutOfSync, or Unknown
  - **Health status** (before/after): Shows if application is Healthy, Progressing, Degraded, etc.
  - **Sync revision** (before/after): Git commit hash that was last synced
- **Change indicators**: Visual indicators (🔄) for changed status, (✓) for unchanged
- **Summary**: Clear indication of what changed and whether any updates were detected
- **Tips**: Helpful guidance on next steps

## Use Cases

1. **Resolve Stale Sync Status**: Fix "OutOfSync" status that's actually in sync
2. **Update After Git Push**: Force ArgoCD to detect new commits after pushing to Git
3. **Troubleshoot Stuck Applications**: Resolve applications that appear frozen or not updating
4. **Verify Configuration Changes**: Check if Git changes are detected by ArgoCD
5. **Fix Cache Issues**: Clear stale cached application state
6. **Pre-Sync Verification**: Check current state before triggering a sync
7. **Detect New Revisions**: Find out if new commits are available in Git

## Examples

### Example 1: Basic Refresh

```json
{
  "application_name": "guestbook"
}
```

### Example 2: Normal Refresh

```json
{
  "application_name": "my-app",
  "refresh_type": "normal"
}
```

### Example 3: Hard Refresh with Namespace

```json
{
  "application_name": "production-api",
  "refresh_type": "hard",
  "app_namespace": "argocd",
  "project": "production"
}
```

## Response Example - Changes Detected

```
🔄 Refreshed Application 'guestbook'
════════════════════════════════════════════════════════════════════════════════

Refresh Type: hard
Repository: https://github.com/argoproj/argocd-example-apps
Target Revision: HEAD

────────────────────────────────────────────────────────────────────────────────

📊 Status Comparison:

🔄  Sync Status:
   Before: OutOfSync
   After:  Synced
   ➜ Changed!

✓  Health Status:
   Before: Healthy
   After:  Healthy
   ➜ No change

────────────────────────────────────────────────────────────────────────────────

✅ Refresh completed - Application state was updated

Changes detected in: sync status

💡 Tips:
   - Refresh does not modify cluster resources, only ArgoCD's cache
   - Use 'hard' refresh to force re-fetch from Git repository
   - If sync status changed to 'OutOfSync', use 'sync_application' to deploy
```

## Response Example - No Changes

```
🔄 Refreshed Application 'stable-app'
════════════════════════════════════════════════════════════════════════════════

Refresh Type: hard
Repository: https://github.com/example/repo
Target Revision: main

────────────────────────────────────────────────────────────────────────────────

📊 Status Comparison:

✓  Sync Status:
   Before: Synced
   After:  Synced
   ➜ No change

✓  Health Status:
   Before: Healthy
   After:  Healthy
   ➜ No change

────────────────────────────────────────────────────────────────────────────────

✅ Refresh completed - No changes detected

The application state in ArgoCD matches the Git repository.

💡 Tips:
   - Refresh does not modify cluster resources, only ArgoCD's cache
   - Use 'hard' refresh to force re-fetch from Git repository
   - If sync status changed to 'OutOfSync', use 'sync_application' to deploy
```

## Error Handling

The tool will return an error in the following cases:

- **Client not initialized**: ArgoCD client is not properly configured
- **Application not found**: The specified application doesn't exist
- **Invalid parameters**: Missing required parameters or invalid values
- **API errors**: ArgoCD API returns an error (e.g., unauthorized, forbidden, server error)
- **Network errors**: Unable to connect to ArgoCD server or Git repository

## Best Practices

1. **Use Hard Refresh for Git Updates**: Always use "hard" refresh after pushing to Git
   ```json
   {
     "application_name": "my-app",
     "refresh_type": "hard"
   }
   ```

2. **Troubleshooting Workflow**: When app appears stuck:
   ```
   1. refresh_application → Force cache update
   2. Check status changes
   3. If OutOfSync, run sync_application
   ```

3. **Before Syncing**: Refresh before sync to ensure latest state
   ```
   1. refresh_application → Get latest from Git
   2. Verify status
   3. sync_application → Deploy if needed
   ```

4. **Regular Monitoring**: Refresh periodically to detect Git changes
   - ArgoCD auto-refreshes every 3 minutes by default
   - Manual refresh provides immediate update

5. **Safe Operation**: Refresh is always safe to run
   - Read-only operation
   - Does not modify cluster resources
   - Only updates ArgoCD's cache

## Refresh Type Comparison

| Refresh Type | Description | Use Case | Performance |
|--------------|-------------|----------|-------------|
| **hard** (default) | Forces re-fetch from Git repository | After Git push, troubleshooting, ensure latest | Slower (Git fetch) |
| **normal** | Uses ArgoCD's cache | Quick status update, regular checks | Faster (cached) |

**Recommendation**: Use "hard" refresh for most cases to ensure you get the latest state from Git.

## Common Scenarios

### Scenario 1: After Git Push
**Problem**: Just pushed changes to Git, but ArgoCD doesn't show OutOfSync

**Solution**:
```json
{
  "application_name": "my-app",
  "refresh_type": "hard"
}
```

**Expected Result**: Sync status changes from "Synced" to "OutOfSync" after detecting new commits

### Scenario 2: Stuck Application
**Problem**: Application shows "Progressing" for too long

**Solution**:
```json
{
  "application_name": "stuck-app",
  "refresh_type": "hard"
}
```

**Expected Result**: Status updates to actual current state (Healthy or Degraded)

### Scenario 3: False OutOfSync
**Problem**: Application shows "OutOfSync" but cluster and Git match

**Solution**:
```json
{
  "application_name": "false-outofsync",
  "refresh_type": "hard"
}
```

**Expected Result**: Sync status changes from "OutOfSync" to "Synced"

### Scenario 4: Detect New Release
**Problem**: Want to check if new commits are available

**Solution**:
```json
{
  "application_name": "production-app",
  "refresh_type": "hard"
}
```

**Expected Result**:
- If new commits: Revision changes, status becomes OutOfSync
- If no new commits: No changes detected

## Workflow Examples

### Pre-Deployment Workflow
```
1. refresh_application(application_name: "api-service", refresh_type: "hard")
   → Force latest Git fetch

2. Check response:
   - If OutOfSync detected: New changes available
   - If Synced: Already up to date

3. If OutOfSync:
   → sync_application(application_name: "api-service")
```

### Troubleshooting Workflow
```
1. User reports: "App not updating"

2. refresh_application(application_name: "user-app", refresh_type: "hard")
   → Force cache refresh

3. Analyze changes:
   - Sync status changed? → Git has updates
   - Health status changed? → Deployment progressed
   - No changes? → App is actually current

4. Take action based on results
```

### Git Update Detection Workflow
```
1. Developer pushes to main branch

2. refresh_application(application_name: "dev-app", refresh_type: "hard")
   → Detect new commits immediately

3. Check revision change:
   - Before: abc123
   - After: def456
   → New revision detected

4. Decide on deployment
```

## Related Tools

- `get_application`: Get current application status (includes refresh parameter)
- `sync_application`: Deploy changes after refresh detects OutOfSync
- `get_application_history`: View deployment history
- `resource_tree`: See detailed resource state after refresh

## Important Notes

1. **Read-Only Operation**: Refresh never modifies cluster resources
   - Safe to run anytime
   - Only updates ArgoCD's internal cache
   - Does not trigger deployments

2. **Not a Deployment**: Refresh ≠ Sync
   - Refresh: Update ArgoCD's view of desired state from Git
   - Sync: Apply desired state to cluster
   - Always refresh before sync for latest state

3. **Auto-Refresh**: ArgoCD auto-refreshes applications periodically
   - Default interval: 3 minutes
   - Manual refresh provides immediate update
   - Useful when you can't wait for automatic refresh

4. **Performance**: Hard refresh requires Git fetch
   - May take longer than normal refresh
   - Still faster than triggering a full sync
   - Recommended for most use cases

5. **Available in Read-Only Mode**: This tool is available even when `ARGOCD_READ_ONLY=true`
   - Does not modify state
   - Safe for read-only environments
   - Useful for monitoring and troubleshooting

## API Endpoint

```
GET /api/v1/applications/{name}?refresh=hard
```

The refresh is triggered by including the `refresh` query parameter in the GET application request.

## Context Optimization

This tool is optimized for LLM context efficiency by:
- Showing only changed fields prominently
- Using visual indicators (🔄, ✓) for quick scanning
- Providing clear before/after comparison
- Summarizing what changed in natural language
- Offering both human-readable and JSON formats

## Read-Only Tool

This is a read-only tool that does not modify any cluster resources. It only updates ArgoCD's cached view of the application. It is safe to use in production environments and is available in read-only mode.
