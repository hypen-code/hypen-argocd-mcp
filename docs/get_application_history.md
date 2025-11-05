# Get Application History

## Overview

The `get_application_history` tool retrieves the complete deployment history for an ArgoCD application. It returns a list of all deployments including revision information, timestamps, source details, and who initiated each deployment. **This tool is essential for rollback operations as it provides the history IDs required by the `rollback_application` tool.**

## Tool Name
`get_application_history`

## Description
Get deployment history for an ArgoCD application. Returns a list of all deployments including revision, timestamp, source information, and who initiated each deployment. Essential for rollback operations (provides history IDs), auditing deployments, and understanding application changes over time. History is sorted newest first.

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `application_name` | string | The name of the ArgoCD application |

### Optional Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `app_namespace` | string | The namespace of the ArgoCD application (if not in default ArgoCD namespace) |
| `project` | string | The ArgoCD project identifier |

## Response

The tool returns an optimized summary with:
- **Application name**: The name of the application
- **Total entries**: Total number of deployment history entries
- **History entries** (sorted newest first, limited to 20 most recent):
  - **History ID**: The unique identifier for this deployment (required for rollback)
  - **Revision**: Git commit hash (shortened to 8 characters for display, full hash included)
  - **Deployed at**: Timestamp when the deployment completed
  - **Deploy duration**: Time taken to deploy (if available)
  - **Initiated by**: Username who triggered the deployment or "Automated"
  - **Automated indicator**: 🤖 emoji for automated deployments
  - **Source repository**: Git repository URL
  - **Source path/chart**: Path in repository or Helm chart name
  - **Target revision**: Branch, tag, or commit that was deployed
- **Current deployment marker**: 👉 emoji for the currently deployed version

## Use Cases

1. **Find History ID for Rollback**: The most critical use case - get history IDs to use with `rollback_application`
2. **Audit Deployments**: Track who deployed what and when
3. **Understand Deployment Timeline**: See the progression of deployments over time
4. **Identify Automated vs Manual Deployments**: Distinguish between auto-sync and manual deployments
5. **Track Source Changes**: See which Git revisions were deployed and when
6. **Troubleshoot Issues**: Correlate issues with specific deployments
7. **Verify Deployment Status**: Confirm the current deployment and previous versions

## Examples

### Example 1: Get History for an Application

```json
{
  "application_name": "guestbook"
}
```

### Example 2: Get History with Namespace

```json
{
  "application_name": "my-app",
  "app_namespace": "argocd"
}
```

### Example 3: Get History for Project-Specific App

```json
{
  "application_name": "production-api",
  "app_namespace": "argocd",
  "project": "production"
}
```

## Response Example

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

   3. History ID: 3
   Revision: abc123de (abc123def456789012345678901234567890)
   Deployed: 2025-01-03T10:00:00Z
   Duration: from 2025-01-03T09:59:00Z to 2025-01-03T10:00:00Z
   Initiated by: admin
   Repository: https://github.com/argoproj/argocd-example-apps
   Path: guestbook
   Target Revision: main

────────────────────────────────────────────────────────────────────────────────

💡 Tips:
   - Use the History ID with 'rollback_application' to revert to a previous version
   - Current deployment is marked with 👉
   - 🤖 indicates automated deployments
```

## Response with No History

```
📜 Deployment History for 'new-app'
════════════════════════════════════════════════════════════════════════════════

Total deployments: 0

⚠️  No deployment history available for this application.
   This could mean the application has never been synced.
```

## Error Handling

The tool will return an error in the following cases:

- **Client not initialized**: ArgoCD client is not properly configured
- **Application not found**: The specified application doesn't exist
- **Invalid parameters**: Missing required parameters or invalid values
- **API errors**: ArgoCD API returns an error (e.g., unauthorized, forbidden, server error)
- **Network errors**: Unable to connect to ArgoCD server

## Best Practices

1. **Use for Rollback Workflow**: Always check history before rollback
   - Get history IDs from this tool
   - Use history ID with `rollback_application`
   - Example workflow:
     ```
     1. get_application_history → Get history IDs
     2. Identify the version to rollback to
     3. rollback_application with history ID
     ```

2. **Audit and Compliance**: Track deployment history for audit trails
   - Who deployed what and when
   - Automated vs manual deployments
   - Source revisions and branches

3. **Troubleshooting**: Correlate issues with deployments
   - Find when a problematic deployment occurred
   - Identify which revision caused issues
   - Get history ID to rollback

4. **Deployment Verification**: Confirm current deployment
   - The first entry (marked with 👉) is the current deployment
   - Verify revision matches expected state

5. **Historical Analysis**: Understand deployment patterns
   - How often is the app deployed?
   - Who deploys most frequently?
   - Are deployments automated or manual?

## Workflow Examples

### Rollback Workflow

```
Step 1: Get deployment history
→ get_application_history(application_name: "my-app")

Step 2: Identify the deployment to rollback to
→ History shows:
   - ID 5 (Current): Broken deployment
   - ID 4: Last working version
   - ID 3: Previous version

Step 3: Rollback to last working version
→ rollback_application(application_name: "my-app", id: 4)
```

### Audit Workflow

```
Step 1: Get deployment history
→ get_application_history(application_name: "production-api")

Step 2: Analyze history
→ Review who deployed what and when
→ Identify unauthorized or suspicious deployments
→ Generate audit report
```

### Troubleshooting Workflow

```
Step 1: User reports issue started around 2025-01-05 10:00
→ get_application_history(application_name: "api-service")

Step 2: Find deployment around that time
→ History shows deployment at 2025-01-05 10:05:00
→ Revision: abc123, deployed by: automated

Step 3: Investigate that specific revision
→ revision_metadata(application_name: "api-service", revision: "abc123")
→ Check commit message and changes

Step 4: Rollback if needed
→ rollback_application(application_name: "api-service", id: 4)
```

## Related Tools

- `rollback_application`: Rollback to a specific history ID from this tool
- `revision_metadata`: Get detailed information about a specific revision
- `get_application`: Get current application status and configuration
- `sync_application`: Trigger a new deployment (creates new history entry)

## Implementation Details

### Data Source
This tool retrieves history from the application's status field (`status.history[]`), which is maintained by ArgoCD for each application. The history is populated whenever a sync operation completes.

### Context Optimization
The tool is optimized for LLM context efficiency by:
- Showing only the 20 most recent deployments
- Shortening revision hashes to 8 characters (full hash available in JSON)
- Summarizing source information
- Providing visual indicators (👉, 🤖) for quick scanning
- Offering both human-readable and JSON formats

### Sorting
History entries are always sorted by History ID in descending order (newest first), so:
- The first entry is always the current deployment
- Earlier entries are older deployments
- History ID 1 is typically the oldest deployment

## Read-Only Tool

This is a read-only tool that does not modify any resources. It is safe to use in production environments and is available in read-only mode.

## API Endpoint

This tool uses the standard application GET endpoint:
```
GET /api/v1/applications/{name}
```

The history is extracted from the `status.history` field of the application response.
