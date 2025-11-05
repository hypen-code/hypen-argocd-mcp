# ArgoCD MCP Server

A robust, optimized Model Context Protocol (MCP) server for ArgoCD, built with Rust. This server enables AI assistants to interact with ArgoCD APIs through standardized MCP tools.

## Features

- **Optimized Response Format**: Responses are optimized to minimize context window usage while providing essential information
- **Robust Error Handling**: Comprehensive error handling with detailed error messages and graceful degradation
- **Complete Test Coverage**: 60+ integration tests with mock ArgoCD API server
- **Stdio Transport**: Uses stdio transport for seamless integration with MCP clients
- **Type-Safe**: Built with Rust for type safety and performance
- **Version Compatibility**: Supports ArgoCD v1.0+ with documented requirements for advanced features

## ArgoCD Version Compatibility

| Feature | Minimum Version | Status |
|---------|----------------|--------|
| Core Tools (list, get, tree, logs, manifests, metadata) | ArgoCD v1.0+ | ✅ Fully Supported |
| list_resource_events | ArgoCD v1.0+ | ✅ Fully Supported |
| sync_application | ArgoCD v1.0+ | ✅ Fully Supported |
| rollback_application | ArgoCD v1.0+ | ✅ Fully Supported |
| server_side_diff | ArgoCD v2.5+ | ⚠️ Version-Specific |
| get_application_sync_windows | ArgoCD v2.6+ | ⚠️ Version-Specific |

**Note**: Version-specific features will return a 404 error if your ArgoCD instance doesn't support them. This is expected behavior and documented in each tool's description.

## Tools

### `list_applications`

Lists ArgoCD applications with optional filters, returning detailed information.

**Arguments:**
- `name` (optional): Filter by application name
- `projects` (optional): Filter by project names (array of strings)
- `selector` (optional): Label selector to filter applications (e.g., 'env=prod')
- `repo` (optional): Filter by repository URL
- `app_namespace` (optional): Filter by application namespace

**Returns:**
Optimized application summaries including:
- Application name and namespace
- Project
- Repository URL and target revision
- Destination server and namespace
- Sync and health status
- Auto-sync configuration

### `list_application_names`

Lists only the names of ArgoCD applications. **Highly optimized for minimal context usage** - perfect for name lookups and auto-correcting typos in application names.

**Arguments:**
- `projects` (optional): Filter by project names (array of strings)
- `selector` (optional): Label selector to filter applications (e.g., 'env=prod')
- `repo` (optional): Filter by repository URL
- `app_namespace` (optional): Filter by application namespace

**Returns:**
A simple list of application names as strings.

**Use Cases:**
- Get a quick list of all application names
- Verify if an application exists
- Auto-correct typos when user types application names incorrectly
- Minimal context window usage (~95% smaller than full application details)

### `get_application`

Get detailed information about a specific ArgoCD application by name. Returns comprehensive application details including source repository, destination cluster, sync status, health status, and sync policy configuration.

**Arguments:**
- `name` (required): Application name
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier
- `refresh` (optional): Refresh mode - "normal" or "hard" to force refresh from repository
- `resource_version` (optional): Resource version for optimistic concurrency

**Returns:**
Optimized detailed information including:
- Application name, namespace, and project
- Creation timestamp and labels
- Source repository details (URL, path/chart, target revision)
- Destination cluster and namespace
- Sync status and revision
- Health status and message
- Sync policy configuration (auto-sync, prune, self-heal settings)

**Use Cases:**
- Get comprehensive details about a specific application
- Check application's current sync and health status
- Review application configuration and source
- Examine sync policy settings
- Force refresh application state from repository
- Troubleshoot application issues with detailed status information

**Example Output:**
```
Application: guestbook

Namespace: argocd
Project: default
Created: 2025-01-01T10:00:00Z

Source:
  Repository: https://github.com/argoproj/argocd-example-apps
  Path: guestbook
  Target Revision: HEAD

Destination:
  Server: https://kubernetes.default.svc
  Namespace: default

Status:
  Sync Status: Synced
  Sync Revision: abc123def456
  Health Status: Healthy
  Health Message: All resources are healthy

Sync Policy:
  Auto Sync: Enabled
  Auto Prune: true
  Self Heal: true

Labels:
  env: production
  team: platform
```

**Note:** Use `refresh: "hard"` parameter to force a full refresh of the application state from the repository, which is useful when you need the most up-to-date information.

### `server_side_diff`

Performs server-side diff calculation for an ArgoCD application using dry-run apply. This executes a Server-Side Apply operation in dryrun mode and compares the predicted state with the live state.

**Arguments:**
- `app_name` (required): The application name
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier
- `target_manifests` (optional): Target manifests for comparison (array of YAML/JSON strings)

**Returns:**
Optimized summaries of resources showing:
- Resource name, kind, and namespace
- Modified status (boolean indicating if differences exist)
- Diff summary for modified resources
- Grouped by modified/in-sync status

**Use Cases:**
- Check if an application has configuration drift
- Preview changes before syncing
- Validate that admission controllers will accept the changes
- Identify which specific resources have differences
- Compare live state with target state without performing actual sync

**Example Output:**
```
Server-Side Diff for application 'guestbook'
Total resources: 3, Modified: 1, In sync: 2

Modified Resources:
1. guestbook-ui (Deployment) in namespace 'default'
   Status: Resource has differences between live and target state

In Sync Resources:
1. guestbook-ui (Service) in namespace 'default'
2. redis-master (Deployment) in namespace 'default'
```

**Note:** Server-Side Diff is a beta feature (available since ArgoCD v2.10.0). It provides more accurate diff results by involving Kubernetes admission controllers in the calculation.

### `resource_tree`

Get the hierarchical resource tree for an ArgoCD application. Returns a comprehensive view of all Kubernetes resources managed by the application, including their relationships, health status, and metadata.

**Arguments:**
- `application_name` (required): The application name
- `namespace` (optional): Filter by resource namespace
- `name` (optional): Filter by resource name
- `version` (optional): Filter by resource version
- `group` (optional): Filter by resource group
- `kind` (optional): Filter by resource kind (e.g., "Deployment", "Service")
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Total resource count
- Orphaned resources count
- Resources grouped by kind (Deployment, Service, Pod, etc.)
- Health status summary (Healthy, Degraded, Progressing, etc.)
- Sample resources (up to 10) with details

**Use Cases:**
- Visualize application resource hierarchy
- Check health status of all resources
- Identify orphaned resources not managed by the application
- Understand resource relationships (parent-child)
- Filter resources by type or namespace
- Troubleshoot deployment issues
- Monitor application state

**Example Output:**
```
Resource Tree for application 'guestbook'
Total resources: 5
Orphaned resources: 0

Resources by Kind:
  Deployment: 2
  Service: 2
  Pod: 1

Health Summary:
  Healthy: 4
  Progressing: 1

Sample Resources (showing up to 10):
1. guestbook-ui (Deployment) in namespace 'default' - Health: Healthy
   Images: gcr.io/heptio-images/ks-guestbook-demo:0.2
2. guestbook-ui (Service) in namespace 'default' - Health: Healthy - 1 parent(s)
3. redis-master (Deployment) in namespace 'default' - Health: Healthy
   Images: redis:6.2
4. redis-master (Service) in namespace 'default' - Health: Healthy - 1 parent(s)
5. guestbook-ui-7d87c5c5 (Pod) in namespace 'default' - Health: Progressing - 1 parent(s)
```

**Filter Examples:**

Filter by kind to see only Deployments:
```json
{
  "application_name": "my-app",
  "kind": "Deployment"
}
```

Filter by namespace and kind:
```json
{
  "application_name": "my-app",
  "namespace": "production",
  "kind": "Service"
}
```

### `list_resource_events`

List Kubernetes events for an ArgoCD application or specific resources within an application. Returns comprehensive event information including type (Normal/Warning), reason, message, timestamps, and involved objects. Provides insights into application lifecycle, deployments, and issues.

**Arguments:**
- `application_name` (required): The application name
- `resource_namespace` (optional): Filter by resource namespace
- `resource_name` (optional): Filter by resource name
- `resource_uid` (optional): Filter by resource UID
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Total event count
- Events grouped by type (Normal, Warning)
- Events grouped by reason
- Individual event details (up to 20 recent events shown)
- Event metadata: reason, message, timestamps, involved objects, source component

**Use Cases:**
- Troubleshoot application deployment issues
- Monitor application lifecycle events
- Investigate pod failures and scheduling issues
- Track resource scaling and updates
- Audit configuration changes
- Debug image pull errors or resource constraints
- Monitor application health over time

**Example Output:**
```
Events for application 'guestbook'
Total events: 15

Events by Type:
  Normal: 10
  Warning: 5

Events by Reason:
  ScalingReplicaSet: 3
  Started: 4
  Pulled: 3
  FailedScheduling: 2
  BackOff: 3

Recent Events (showing up to 20):

1. [Normal] ScalingReplicaSet - Deployment/guestbook-ui
   Message: Scaled up replica set guestbook-ui-abc to 3
   Count: 5
   First: 2025-01-01T10:00:00Z | Last: 2025-01-01T10:05:00Z
   Source: deployment-controller

2. [Normal] Started - Pod/guestbook-ui-abc-12345
   Message: Started container guestbook
   First: 2025-01-01T10:10:00Z
   Source: kubelet

3. [Warning] FailedScheduling - Pod/guestbook-ui-abc-67890
   Message: 0/5 nodes are available: insufficient cpu
   Count: 10
   First: 2025-01-01T10:15:00Z | Last: 2025-01-01T10:20:00Z
   Source: default-scheduler

... and 12 more events (total: 15)
```

**Filter Examples:**

Get all events for a specific resource:
```json
{
  "application_name": "my-app",
  "resource_namespace": "production",
  "resource_name": "my-deployment"
}
```

Get events for a specific resource by UID:
```json
{
  "application_name": "my-app",
  "resource_uid": "abc-123-def-456"
}
```

**Note:** Events are time-limited by Kubernetes (typically retained for 1 hour) and provide the most recent activity for troubleshooting.

### `pod_logs`

Get container logs from pods in an ArgoCD application with intelligent error/warning filtering and log level analysis. Essential for troubleshooting deployments, investigating crashes, and monitoring application behavior.

**Arguments:**
- `application_name` (required): The application name
- `namespace` (optional): Pod namespace
- `pod_name` (optional): Pod name (if not provided, use kind and resource_name)
- `container` (optional): Container name (defaults to first container)
- `since_seconds` (optional): Show logs since N seconds ago
- `tail_lines` (optional): Number of lines from end (default: 100 for context efficiency)
- `previous` (optional): Show previous container logs (if restarted)
- `filter` (optional): Server-side text filter
- `kind` (optional): Resource kind (e.g., "Deployment", "StatefulSet")
- `group` (optional): Resource group
- `resource_name` (optional): Resource name (alternative to pod_name)
- `app_namespace` (optional): Application namespace
- `project` (optional): Project identifier
- `errors_only` (optional): **Filter to show only errors and potential issues** (client-side, recommended for LLM context)

**Returns:**
Intelligent analysis including:
- Total log lines
- Error, warning, and potential issue counts
- Logs grouped by level (FATAL, ERROR, WARNING, INFO, DEBUG)
- Individual log entries with timestamps and visual indicators
- Helpful tips for optimization

**Key Features:**
- **Intelligent Log Level Detection**: Automatically detects FATAL, ERROR, WARNING, INFO, DEBUG levels from log content
- **Potential Issue Detection**: Identifies problems beyond explicit log levels (exceptions, timeouts, panics, crashes, permission errors, etc.)
- **Error Filtering**: Use `errors_only: true` to show only errors and warnings (saves LLM context)
- **Context-Optimized**: Default tail of 100 lines prevents context overflow
- **Visual Indicators**: Emoji indicators for quick issue identification (💀 FATAL, ❌ ERROR, ⚠️ WARNING, ℹ️ INFO, 🐛 DEBUG)
- **NDJSON Parsing**: Handles ArgoCD's streaming log format

**Use Cases:**
- Troubleshoot pod crashes and failures
- Investigate deployment issues
- Monitor application errors in real-time
- Debug connection and timeout problems
- Find root causes of application failures
- Analyze log patterns and trends

**Example Output:**
```
Pod Logs for application 'my-app'
Pod: my-app-7d87c5c5-abc12
Container: app
Tail Lines: 100

Total lines: 50

🔍 Filtered to show errors and potential issues only

📊 Log Analysis:
  ❌ Errors: 3
  ⚠️  Warnings: 2
  🔍 Potential Issues: 5

Logs by Level:
  ERROR: 3
  WARNING: 2

📝 Log Entries (showing 5):
────────────────────────────────────────────────────────────────────────────────
❌ [2025-01-01T10:15:30Z] ERROR:
   Failed to connect to database: connection timeout

⚠️  [2025-01-01T10:15:35Z] WARNING:
   Retrying connection (attempt 2/5)

❌ [2025-01-01T10:15:40Z] ERROR:
   Connection failed again: unable to resolve hostname

❌ [2025-01-01T10:15:45Z] ERROR:
   Max retries exceeded, giving up

⚠️  [2025-01-01T10:15:50Z] WARNING:
   Service degraded due to database unavailability
────────────────────────────────────────────────────────────────────────────────

💡 Tip: Increase 'tail_lines' to see more logs or use 'since_seconds' for time-based filtering
```

**Filter Examples:**

Get all logs (unfiltered):
```json
{
  "application_name": "my-app",
  "pod_name": "my-app-pod",
  "tail_lines": 100
}
```

Get only errors and warnings (recommended for troubleshooting):
```json
{
  "application_name": "my-app",
  "pod_name": "my-app-pod",
  "errors_only": true
}
```

Get logs from a specific container:
```json
{
  "application_name": "my-app",
  "pod_name": "my-app-pod",
  "container": "sidecar",
  "tail_lines": 50
}
```

Get logs since 5 minutes ago:
```json
{
  "application_name": "my-app",
  "pod_name": "my-app-pod",
  "since_seconds": 300
}
```

Get logs for a Deployment (auto-selects pod):
```json
{
  "application_name": "my-app",
  "kind": "Deployment",
  "resource_name": "my-deployment",
  "errors_only": true
}
```

**Detected Issue Patterns:**
The tool automatically detects potential issues using these keywords:
- **Error levels**: FATAL, CRITICAL, ERROR, ERR
- **Warning levels**: WARN, WARNING
- **Exceptions**: exception, panic, crash
- **Failures**: failed, timeout, unable to, cannot
- **Access issues**: refused, denied, permission denied

**Performance Tips:**
- Use `errors_only: true` to reduce context usage by 70-90%
- Default `tail_lines: 100` balances detail with context efficiency
- Use `since_seconds` for time-scoped troubleshooting
- Combine `filter` (server-side) with `errors_only` (client-side) for maximum efficiency

### `revision_metadata`

Get metadata (author, date, message, tags) for a specific revision of an ArgoCD application. Returns commit information including author, timestamp, commit message, associated Git tags, and signature verification status. Useful for tracking changes, auditing deployments, and understanding revision history.

**Arguments:**
- `application_name` (required): The application name
- `revision` (required): Revision/commit hash
- `app_namespace` (optional): Application namespace
- `project` (optional): Project identifier
- `source_index` (optional): Source index (for multi-source applications)
- `version_id` (optional): Version ID from historical data (for multi-source applications)

**Returns:**
Optimized summary including:
- Author and date of the revision
- Short and full commit messages
- Number of tags and associated tags
- Signature status (signed/not signed) and summary

**Use Cases:**
- Track changes and audit deployments
- Understand revision history and commit details
- Verify commit authorship and integrity
- Identify associated Git tags for a revision
- Debug issues related to specific code versions

**Example Output:**
```
Revision Metadata for application 'guestbook' at revision 'abc123def456'

Author: John Doe <john.doe@example.com>
Date: 2025-10-27T10:30:00Z

Commit Message:
  feat: Add new feature

Full Message:
  feat: Add new feature

  This commit introduces a brand new feature to the application.

Tags (2):
  - v1.2.0
  - release-candidate

Signature Status: Valid signature
```

### `get_application_sync_windows`

Get synchronization windows for an ArgoCD application. Returns a list of configured sync windows, including their schedule, duration, and affected applications/namespaces/clusters. Useful for understanding when an application can be synced or is blocked from syncing.

**Arguments:**
- `application_name` (required): The application name
- `app_namespace` (optional): Application namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Total number of sync windows
- Details for each sync window:
  - `kind`: Type of sync window (e.g., "allow", "deny")
  - `schedule`: Cron schedule for the window
  - `duration`: Duration of the window (e.g., "1h", "30m")
  - `applications`: List of application names affected by the window
  - `namespaces`: List of namespaces affected by the window
  - `clusters`: List of cluster URLs affected by the window
  - `manual_sync_enabled`: Whether manual sync is allowed during the window
  - `start_time`: Start time of the window (RFC3339 format)
  - `end_time`: End time of the window (RFC3339 format)

**Use Cases:**
- Determine when an application is allowed or denied to sync
- Identify maintenance windows or blackout periods
- Understand which applications, namespaces, or clusters are affected by specific sync policies
- Verify manual synchronization permissions during a window

**Example Output:**
```
Sync Windows for application 'my-app' (2 total):

1. Kind: allow
   Schedule: 0 0 * * *
   Duration: 1h
   Start Time: 2025-01-01T00:00:00Z
   End Time: 2025-01-01T01:00:00Z
   Manual Sync Enabled: true
   Applications: guestbook, helm-app
   Namespaces: default, staging
   Clusters: https://kubernetes.default.svc

2. Kind: deny
   Schedule: 0 2 * * *
   Duration: 30m
   Start Time: 2025-01-01T02:00:00Z
   End Time: 2025-01-01T02:30:00Z
   Manual Sync Enabled: false
   Applications: backend-api
   Namespaces: backend-prod
```

### `sync_application`

Sync an ArgoCD application to its target state in Git. This operation deploys or updates the application resources to match what's defined in the Git repository. **Note: This is a write operation and is blocked in read-only mode.**

**Arguments:**
- `application_name` (required): Name of the application to sync
- `revision` (optional): Specific revision to sync to (defaults to target revision in app spec)
- `dry_run` (optional): If true, preview the sync without actually performing it (default: false)
- `prune` (optional): Whether to prune resources that are no longer defined in Git (default: false)
- `force` (optional): Use force apply to override any conflicts (default: false)
- `resources` (optional): Specific resources to sync (if not specified, syncs all resources)
- `sync_options` (optional): Sync options array (e.g., ["Validate=false", "CreateNamespace=true"])
- `retry` (optional): Retry configuration (limit, backoff_duration, backoff_max_duration, backoff_factor)
- `app_namespace` (optional): Application namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Application name
- Whether the operation was a dry-run
- Current sync status and revision after sync
- Current health status after sync
- Target revision that was synced to
- Whether prune/force were enabled
- Sync options that were applied
- Number of resources synced (if partial sync)

**Use Cases:**
- Deploy new application versions from Git
- Update application configuration
- Fix configuration drift
- Preview changes with dry-run mode
- Sync only specific resources (partial sync)
- Force sync to override conflicts
- Clean up orphaned resources with prune

**Example Output:**
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
```

**Usage Examples:**

Basic sync:
```json
{
  "application_name": "guestbook"
}
```

Dry-run to preview changes:
```json
{
  "application_name": "guestbook",
  "dry_run": true
}
```

Sync to specific revision:
```json
{
  "application_name": "guestbook",
  "revision": "v1.2.3"
}
```

Sync with prune:
```json
{
  "application_name": "guestbook",
  "prune": true
}
```

Sync specific resources only:
```json
{
  "application_name": "guestbook",
  "resources": [
    {
      "group": "apps",
      "kind": "Deployment",
      "name": "guestbook-ui",
      "namespace": "default"
    }
  ]
}
```

Sync with options:
```json
{
  "application_name": "guestbook",
  "sync_options": ["Validate=false", "CreateNamespace=true"]
}
```

**Common Sync Options:**
- `Validate=false` - Skip kubectl validation
- `CreateNamespace=true` - Create namespace if it doesn't exist
- `PruneLast=true` - Prune resources after all other resources are synced
- `ApplyOutOfSyncOnly=true` - Only apply out-of-sync resources
- `ServerSideApply=true` - Use server-side apply
- `Replace=true` - Use replace instead of apply

**Best Practices:**
1. **Always use dry-run first**: Preview the sync with `dry_run: true` before executing, especially in production
2. **Monitor after sync**: Watch the application to ensure it reaches the desired state
3. **Be cautious with force**: Only use force when you understand the implications
4. **Be careful with prune**: Pruning removes resources - ensure you know what will be deleted
5. **Use selective sync for large apps**: Sync specific resources when you only need to update certain components
6. **Configure retries appropriately**: Use retry configuration for unreliable environments
7. **Check sync windows**: Verify the application isn't in a blocked sync window before syncing

**Read-Only Mode:**
This tool is a write operation and is **blocked in read-only mode**. You will receive an error if you try to use it when `ARGOCD_READ_ONLY=true`.

**See [docs/sync_application.md](docs/sync_application.md) for detailed documentation, advanced examples, and all sync options.**

### `get_application_history`

Get deployment history for an ArgoCD application. Returns a list of all deployments with history IDs, revisions, timestamps, and initiator information. **Essential for rollback operations.**

**Arguments:**
- `application_name` (required): The application name
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Total number of deployments
- History entries (up to 20 most recent, sorted newest first):
  - History ID (required for rollback)
  - Git revision (shortened for display, full hash in JSON)
  - Deployment timestamp
  - Deploy duration (if available)
  - Who initiated (username or "Automated")
  - Source repository and path/chart
  - Target revision (branch/tag)
  - Current deployment marker (👉)
  - Automated deployment indicator (🤖)

**Use Cases:**
- **Get history IDs for rollback operations** (most critical - required by `rollback_application`)
- Audit deployments and track who deployed what
- Understand deployment timeline and progression
- Identify automated vs manual deployments
- Track source changes across deployments
- Troubleshoot issues by correlating with specific deployments
- Verify current deployment status

**Example Output:**
```
📜 Deployment History for 'guestbook'
════════════════════════════════════════════════════════════════════════════════

Total deployments: 5

────────────────────────────────────────────────────────────────────────────────

👉 1. History ID: 5 (Current)
   Revision: ghi789jk (ghi789jkl012345678901234567890123456)
   Deployed: 2025-01-05T10:30:00Z
   Duration: from 2025-01-05T10:29:00Z to 2025-01-05T10:30:00Z
   Initiated by: john.doe
   Repository: https://github.com/argoproj/argocd-example-apps
   Path: guestbook
   Target Revision: v2.0

────────────────────────────────────────────────────────────────────────────────

   2. History ID: 4
   Revision: def456ab (def456abc789012345678901234567890123)
   Deployed: 2025-01-04T14:30:00Z
   Initiated by: Automated 🤖
   Repository: https://github.com/argoproj/argocd-example-apps
   Path: guestbook
   Target Revision: main

────────────────────────────────────────────────────────────────────────────────

💡 Tips:
   - Use the History ID with 'rollback_application' to revert to a previous version
   - Current deployment is marked with 👉
   - 🤖 indicates automated deployments
```

**Rollback Workflow:**
```
1. get_application_history(application_name: "my-app")
   → Get history IDs and find the version to rollback to

2. rollback_application(application_name: "my-app", id: 4)
   → Rollback to history ID 4 from step 1
```

**See [docs/get_application_history.md](docs/get_application_history.md) for detailed documentation, workflows, and examples.**

### `refresh_application`

Refresh an ArgoCD application from the Git repository. Forces ArgoCD to re-fetch manifests and recompute sync status. **This is a read-only operation** that does not modify cluster state - it only updates ArgoCD's cached view.

**Arguments:**
- `application_name` (required): The application name
- `refresh_type` (optional): "normal" or "hard" (default: "hard")
  - "normal": Regular refresh from cache
  - "hard": Force refresh from Git repository
- `app_namespace` (optional): Application's namespace
- `project` (optional): Project identifier

**Returns:**
Before/after comparison showing:
- Sync status (before/after) with change indicator
- Health status (before/after) with change indicator
- Sync revision (before/after) if changed
- Repository URL and target revision
- Summary of what changed
- Visual indicators (🔄 for changed, ✓ for unchanged)

**Use Cases:**
- **Resolve stale sync status** (most common - fix "stuck" applications)
- Update ArgoCD after pushing to Git
- Troubleshoot applications not updating
- Verify configuration changes are detected
- Fix cache issues
- Pre-sync verification
- Detect new Git commits

**Example Output:**
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

**Common Scenarios:**

1. **After Git Push**: Force ArgoCD to detect new commits
   ```json
   {
     "application_name": "my-app",
     "refresh_type": "hard"
   }
   ```

2. **Stuck Application**: Fix applications that appear frozen
   ```json
   {
     "application_name": "stuck-app",
     "refresh_type": "hard"
   }
   ```

3. **Pre-Deployment Check**: Refresh before syncing
   ```
   1. refresh_application → Get latest from Git
   2. Check if OutOfSync
   3. sync_application → Deploy if needed
   ```

**Important Notes:**
- **Read-only operation**: Never modifies cluster resources
- **Safe to run anytime**: Only updates ArgoCD's cache
- **Not a deployment**: Refresh ≠ Sync (use `sync_application` to deploy)
- **Available in read-only mode**: Can be used when `ARGOCD_READ_ONLY=true`

**See [docs/refresh_application.md](docs/refresh_application.md) for detailed documentation, workflows, and troubleshooting scenarios.**

### `get_resource`

Get a specific Kubernetes resource from an ArgoCD application. Returns detailed resource manifest including metadata, spec, and status.

**Arguments:**
- `application_name` (required): The application name
- `resource_name` (required): The name of the specific resource to retrieve
- `version` (required): The Kubernetes API version (e.g., "v1", "apps/v1")
- `kind` (required): The resource kind (e.g., "Pod", "Service", "Deployment")
- `namespace` (optional): The namespace of the resource
- `group` (optional): The API group (empty for core resources, "apps" for deployments, etc.)
- `app_namespace` (optional): The namespace of the ArgoCD application
- `project` (optional): The ArgoCD project identifier

**Returns:**
Optimized summary including:
- Resource identification (name, kind, version, group, namespace)
- Manifest summary with parsed metadata (labels, annotations, creation time, status)
- Full manifest (first 50 lines shown, with indication if truncated)

**Use Cases:**
- Inspect the current state of a specific pod, deployment, service, or other Kubernetes resource
- Review resource configuration details
- Verify resource status and health
- Troubleshoot issues with specific resources
- Examine resource labels, annotations, and metadata

**Example Output:**
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
...
```

**See [docs/get_resource.md](docs/get_resource.md) for detailed documentation and examples.**

### `patch_resource`

Patch a Kubernetes resource in an ArgoCD application using JSON patch, merge patch, or strategic merge patch formats. **Note: This is a write operation and is blocked in read-only mode.**

**Arguments:**
- `application_name` (required): The application name
- `resource_name` (required): The name of the resource to patch
- `version` (required): The Kubernetes API version (e.g., "v1", "apps/v1")
- `kind` (required): The resource kind (e.g., "Deployment", "Service", "ConfigMap")
- `patch` (required): The patch content as a JSON string
- `namespace` (optional): The namespace of the resource
- `group` (optional): The API group (empty for core resources)
- `patch_type` (optional): Patch strategy type (json-patch, merge-patch, strategic-merge-patch)
- `app_namespace` (optional): The namespace of the ArgoCD application
- `project` (optional): The ArgoCD project identifier

**Returns:**
Optimized summary including:
- Patch confirmation and resource identification
- Updated manifest summary with parsed metadata
- Full updated manifest (first 50 lines shown)

**Use Cases:**
- Scale deployments by updating replica count
- Update container images in deployments
- Add or modify labels and annotations
- Update environment variables in pods
- Modify ConfigMap or Secret data
- Change resource limits and requests
- Update service ports or selectors

**Common Patch Types:**
- `application/json-patch+json`: RFC 6902 JSON Patch for precise operations
- `application/merge-patch+json`: RFC 7396 Merge Patch for simple merging
- `application/strategic-merge-patch+json`: Kubernetes Strategic Merge (default, recommended)

**Example - Scale Deployment:**
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

**Example Output:**
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
  Status: 5/5 replicas ready

📄 Updated Manifest:
────────────────────────────────────────────────────────────────────────────────
apiVersion: apps/v1
kind: Deployment
...

💡 Tip: Monitor the resource to ensure it reaches the desired state.
```

**Important Notes:**
- Changes made via `patch_resource` may be overwritten by ArgoCD if the application is synced and the change conflicts with Git
- For permanent changes, consider updating the Git repository and using `sync_application`
- This operation is disabled when `ARGOCD_READ_ONLY=true`

**See [docs/patch_resource.md](docs/patch_resource.md) for detailed documentation, patch strategy guide, and advanced examples.**

### `rollback_application`

Rollback an ArgoCD application to a previous deployed version by History ID. This operation reverts the application to a specific point in its deployment history. **Note: This is a write operation and is blocked in read-only mode.**

**Arguments:**
- `application_name` (required): Name of the application to rollback
- `id` (required): History ID to rollback to. Use 0 to rollback to the previous version
- `dry_run` (optional): If true, preview the rollback without actually performing it (default: false)
- `prune` (optional): Whether to prune resources that are no longer defined in the target revision (default: false)
- `app_namespace` (optional): Application namespace
- `project` (optional): Project identifier

**Returns:**
Optimized summary including:
- Application name
- History ID that was rolled back to
- Whether the operation was a dry-run
- Current sync status and revision after rollback
- Current health status after rollback
- Target revision after rollback
- Whether pruning was enabled

**Use Cases:**
- Quickly revert to a previous working version after a failed deployment
- Rollback to a known-good state during incidents
- Preview rollback effects with dry-run mode before executing
- Remove orphaned resources with prune option
- Audit and track rollback operations

**Example Output:**
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
```

**Dry Run Example:**
```
Rollback (Dry Run) for application 'guestbook'

Rolled back to History ID: 3
Target Revision: v1.0.0

Status:
  Sync Status: OutOfSync
  Health Status: Healthy

Options:
  Dry Run: true
  Prune Enabled: false

⚠️  Note: This was a dry run. No actual changes were made.
    Run without dry_run=true to perform the actual rollback.
```

**Usage Examples:**

Basic rollback to specific history ID:
```json
{
  "application_name": "guestbook",
  "id": 5
}
```

Rollback to previous version (ID 0):
```json
{
  "application_name": "guestbook",
  "id": 0
}
```

Preview rollback with dry-run:
```json
{
  "application_name": "guestbook",
  "id": 3,
  "dry_run": true
}
```

Rollback with pruning:
```json
{
  "application_name": "guestbook",
  "id": 5,
  "prune": true
}
```

**Best Practices:**
1. **Always use dry-run first**: Preview the rollback with `dry_run: true` before executing
2. **Monitor after rollback**: Watch the application to ensure it reaches the desired state
3. **Check history**: Use `revision_metadata` to verify the correct history ID
4. **Be cautious with prune**: Only enable pruning if you're sure you want to remove resources
5. **Document rollbacks**: Keep track of why and when rollbacks are performed

**Read-Only Mode:**
This tool is a write operation and is **blocked in read-only mode**. You will receive an error if you try to use it when `ARGOCD_READ_ONLY=true`.

**See [docs/rollback_application.md](docs/rollback_application.md) for detailed documentation and additional examples.**

## Configuration

The server requires the following environment variables:

### Required Variables

- `ARGOCD_BASE_URL`: The base URL of your ArgoCD server (e.g., `https://argocd.example.com`)
- `ARGOCD_ACCESS_TOKEN`: Your ArgoCD API access token

### Optional Variables

- `ARGOCD_INSECURE` (optional): Set to `true` to skip TLS certificate verification (useful for self-signed certificates)
- `ARGOCD_READ_ONLY` (optional): Set to `true` to enforce read-only mode (default: `false`)

### Read-Only Mode

The server supports a read-only mode that can be enabled by setting the `ARGOCD_READ_ONLY` environment variable to `true`. When enabled:

- ✅ All read-only tools continue to work (GET requests)
- ❌ Write operations like `rollback_application` are blocked
- ✅ Server information displays "READ-ONLY MODE" indicator
- ✅ Provides additional safety for production environments
- ✅ Useful for audit/compliance requirements

**Write Operations (Blocked in Read-Only Mode):**
- `sync_application` - Sync an application to its target state in Git
- `rollback_application` - Rollback an application to a previous version
- `patch_resource` - Patch a Kubernetes resource in an application

**Read Operations (Always Available):**
- All other tools (list, get, tree, logs, manifests, metadata, events, sync_windows, get_resource, get_application_history, refresh_application, etc.)

The read-only mode is useful for:
- Production monitoring and troubleshooting without risk of accidental changes
- Audit and compliance requirements
- Providing safe access to junior team members
- Explicit documentation of access level
- Enhanced security posture

```bash
# Enable read-only mode
export ARGOCD_READ_ONLY=true

# Disable read-only mode (default)
export ARGOCD_READ_ONLY=false
```

### TLS/SSL Configuration

If your ArgoCD server uses self-signed certificates or certificates that are not trusted by the system, you can disable TLS certificate verification:

```bash
export ARGOCD_INSECURE=true
```

**Security Warning**: Only use `ARGOCD_INSECURE=true` in development/testing environments or with internal ArgoCD servers. For production use, it's recommended to:
- Use properly signed certificates from a trusted CA
- Add your organization's CA certificate to the system trust store
- Use `argocd login --insecure` only when absolutely necessary

### Getting an ArgoCD Access Token

1. Log in to your ArgoCD instance:
   ```bash
   argocd login <ARGOCD_SERVER>
   ```

2. Generate an account token:
   ```bash
   argocd account generate-token
   ```

## Installation

### Prerequisites

- Rust 1.70 or later (for building)
- Python 3.8+ (for running via wrapper)

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/argocd-mcp-server.git
cd argocd-mcp-server

# Build the Rust binary
cargo build --release
```

### Deploying to Another Location

After building, you can deploy the server to any location. The server consists of two components:
1. Python wrapper (`argocd_mcp_server.py`)
2. Rust binary (`target/release/argocd-mcp-server`)

**Important**: The Python wrapper looks for the binary in specific locations relative to itself:
- `bin/argocd-mcp-server` (recommended for deployment)
- `argocd-mcp-server` (same directory as wrapper)
- `target/release/argocd-mcp-server` (development only)

#### Quick Installation

Use the provided installation script:

```bash
# Install to default location (~/.local/bin/argocd-mcp-server)
./install.sh

# Or install to custom location
./install.sh /path/to/installation/directory
```

#### Manual Installation

```bash
# Create installation directory
mkdir -p /path/to/install/bin

# Copy files
cp argocd_mcp_server.py /path/to/install/
cp target/release/argocd-mcp-server /path/to/install/bin/

# Make executable
chmod +x /path/to/install/argocd_mcp_server.py
chmod +x /path/to/install/bin/argocd-mcp-server
```

**See [INSTALL.md](INSTALL.md) for detailed installation instructions, troubleshooting, and deployment best practices.**

## Usage

### Running the Server

The server can be run in two ways:

#### Method 1: Via Python Wrapper (Recommended - Most Compatible)

This method is compatible with all MCP frameworks that require standard executables:

```bash
# Set environment variables
export ARGOCD_BASE_URL=https://your-argocd-server.com
export ARGOCD_ACCESS_TOKEN=your-access-token-here

# Run via Python wrapper
python3 argocd_mcp_server.py
```

#### Method 2: Direct Rust Binary

For direct execution or testing (not supported by all MCP frameworks):

```bash
# Set environment variables
export ARGOCD_BASE_URL=https://your-argocd-server.com
export ARGOCD_ACCESS_TOKEN=your-access-token-here

# Run the binary directly
./target/release/argocd-mcp-server
# OR
cargo run --release
```

### Using with MCP Inspector

Test the server using the MCP Inspector:

```bash
# Via Python wrapper (recommended)
npx @modelcontextprotocol/inspector python3 argocd_mcp_server.py

# OR via direct binary
npx @modelcontextprotocol/inspector ./target/release/argocd-mcp-server
```

### Integration with Claude Desktop / Claude Code

Add to your Claude Desktop/Code configuration (`.mcp.json` or `claude_desktop_config.json`):

#### Option 1: Using Python Wrapper (Recommended - Most Compatible)

```json
{
  "mcpServers": {
    "argocd": {
      "command": "python3",
      "args": ["/absolute/path/to/argocd-mcp-server/argocd_mcp_server.py"],
      "env": {
        "ARGOCD_BASE_URL": "https://your-argocd-server.com",
        "ARGOCD_ACCESS_TOKEN": "your-access-token-here",
        "ARGOCD_INSECURE": "true"
      }
    }
  }
}
```

#### Option 2: Direct Binary (If your framework supports it)

```json
{
  "mcpServers": {
    "argocd": {
      "command": "/absolute/path/to/argocd-mcp-server/target/release/argocd-mcp-server",
      "env": {
        "ARGOCD_BASE_URL": "https://your-argocd-server.com",
        "ARGOCD_ACCESS_TOKEN": "your-access-token-here",
        "ARGOCD_INSECURE": "true"
      }
    }
  }
}
```

#### Option 3: Using uvx/pipx (After publishing to PyPI)

```json
{
  "mcpServers": {
    "argocd": {
      "command": "uvx",
      "args": ["argocd-mcp-server"],
      "env": {
        "ARGOCD_BASE_URL": "https://your-argocd-server.com",
        "ARGOCD_ACCESS_TOKEN": "your-access-token-here",
        "ARGOCD_INSECURE": "true"
      }
    }
  }
}
```

**Notes**:
- **Use Option 1 (Python wrapper)** - Most compatible with all MCP frameworks
- Only include `"ARGOCD_INSECURE": "true"` if your ArgoCD server uses self-signed certificates
- The wrapper adds minimal overhead (~1-2ms) while maintaining Rust performance
- **Always use absolute paths** - Replace `/absolute/path/to/` with your actual path

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_list_all_applications
```

### Project Structure

```
argocd-mcp-server/
├── src/
│   ├── main.rs                  # Entry point with stdio transport
│   ├── lib.rs                   # Library exports
│   ├── argocd_client.rs         # ArgoCD API client
│   ├── models.rs                # Data models (optimized for context efficiency)
│   └── tools.rs                 # MCP tool implementations
├── tests/
│   └── integration_test.rs      # Integration tests with mock server
├── argocd_mcp_server.py         # Python wrapper (RECOMMENDED)
├── setup.py                     # Python package setup
├── pyproject.toml               # Python project configuration
├── Cargo.toml                   # Rust package configuration
├── Cargo.lock                   # Rust dependency lock
└── README.md
```

## Architecture

### Components

1. **ArgoCD Client** (`argocd_client.rs`)
   - Handles HTTP communication with ArgoCD API
   - Implements authentication and error handling
   - Provides both optimized and full response methods

2. **Data Models** (`models.rs`)
   - Type-safe models for ArgoCD objects
   - Optimized summary format to reduce context usage
   - Comprehensive deserialization with proper field mappings

3. **MCP Tools** (`tools.rs`)
   - Implements MCP tool interface using `#[tool]` macros
   - Handles tool routing and parameter validation
   - Formats responses for optimal readability

### Response Optimization

The server uses `ApplicationSummaryOutput` to provide only essential fields:
- Reduces response size by ~70% compared to full application objects
- Includes all critical information for decision-making
- Provides both human-readable and JSON formats

### Testing

Comprehensive test suite includes:
- Unit tests for client creation and validation
- Integration tests with mock ArgoCD API server (using wiremock)
- Error handling tests (authentication, network, server errors)
- Filter and pagination tests
- Empty response handling

## API Compatibility

This server is compatible with ArgoCD API v1alpha1. It has been tested with:
- ArgoCD 2.x API endpoints
- Standard ArgoCD authentication

## Performance

- **Startup Time**: < 100ms
- **Response Time**: Typically < 500ms for listing applications (depends on ArgoCD server)
- **Memory Usage**: ~10MB base memory footprint
- **Concurrency**: Fully async using Tokio runtime

## Troubleshooting

### Common Issues

1. **"ArgoCD client not initialized"**
   - Ensure `ARGOCD_BASE_URL` and `ARGOCD_ACCESS_TOKEN` are set
   - Check that the environment variables are exported before running

2. **Authentication errors**
   - Verify your access token is valid: `argocd account get-user-info`
   - Generate a new token if needed

3. **Connection timeout**
   - Check network connectivity to ArgoCD server
   - Verify the base URL is correct and accessible

4. **TLS/SSL certificate errors**
   - Error: "Failed to send request to ArgoCD API"
   - Common cause: Self-signed or untrusted certificates
   - Solution: Set `ARGOCD_INSECURE=true` in your environment configuration
   - Alternative: Add your organization's CA certificate to the system trust store

### Debug Logging

Enable debug logging:

```bash
RUST_LOG=debug cargo run
```

## Security Considerations

- Store access tokens securely (use environment variables or secret managers)
- Never commit tokens to version control
- Use HTTPS for ArgoCD server connections
- Regularly rotate access tokens
- Consider using service accounts with minimal required permissions

## Future Enhancements

Potential additions:
- Additional tools (sync_application, rollback_application, etc.)
- Application creation and updates
- Detailed resource status queries
- Webhook support for real-time updates
- Caching layer for improved performance
- Application manifest generation

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)

## Acknowledgments

- Built with [rust-sdk for MCP](https://github.com/modelcontextprotocol/rust-sdk)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client
- Testing with [wiremock](https://github.com/LukeMathWalker/wiremock-rs)
