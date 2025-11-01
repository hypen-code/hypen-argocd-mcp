# ArgoCD MCP Server - Example Usage

This document provides examples of using the ArgoCD MCP Server tools with an AI assistant like Claude.

## Prerequisites

1. Set up environment variables:
```bash
export ARGOCD_BASE_URL=https://your-argocd-server.com
export ARGOCD_ACCESS_TOKEN=your-access-token-here
```

2. Run the MCP server:
```bash
cargo run --release
```

## Tool Examples

### 1. List All Applications

**User Query:**
> "Show me all ArgoCD applications"

**Tool Call:**
```json
{
  "tool": "list_applications"
}
```

**Expected Response:**
```
Found 3 application(s):

1. guestbook
   Project: default
   Repository: https://github.com/argoproj/argocd-example-apps
   Target Revision: HEAD
   Destination Server: https://kubernetes.default.svc
   Destination Namespace: default
   Sync Status: Synced
   Health Status: Healthy
   Auto Sync: Enabled

2. helm-app
   Project: platform
   Repository: https://charts.example.com
   Target Revision: 1.2.3
   Destination Server: https://kubernetes.default.svc
   Destination Namespace: staging
   Sync Status: OutOfSync
   Health Status: Progressing

3. backend-api
   Project: backend
   Repository: https://github.com/example/backend-api
   Target Revision: v2.1.0
   Destination Server: https://prod-cluster.example.com
   Destination Namespace: backend-prod
   Sync Status: Synced
   Health Status: Healthy
   Auto Sync: Enabled
```

### 2. Filter Applications by Project

**User Query:**
> "Show me all applications in the 'default' project with 'env=production' label"

**Tool Call:**
```json
{
  "tool": "list_applications",
  "arguments": {
    "projects": ["default"],
    "selector": "env=production"
  }
}
```

**Expected Response:**
```
Found 1 application(s):

1. guestbook
   Project: default
   Repository: https://github.com/argoproj/argocd-example-apps
   Sync Status: Synced
   Health Status: Healthy
```

### 3. List Only Application Names

**User Query:**
> "What are the names of all my ArgoCD applications?"

**Tool Call:**
```json
{
  "tool": "list_application_names"
}
```

**Expected Response:**
```
Found 3 application(s):

1. guestbook
2. helm-app
3. backend-api

--- Application Names Array ---
["guestbook", "helm-app", "backend-api"]
```

**Use Cases:**
- Quick name lookup
- Verify application existence
- Auto-correct typos (e.g., user types "guestbok" but you find "guestbook")
- Minimal token usage for large deployments

### 4. Check Application Drift with Server-Side Diff

**User Query:**
> "Does the guestbook application have any configuration drift?"

**Tool Call:**
```json
{
  "tool": "server_side_diff",
  "arguments": {
    "app_name": "guestbook"
  }
}
```

**Expected Response:**
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

**Interpretation:**
- The application has 3 resources total
- 1 resource (guestbook-ui Deployment) has differences between live and target state
- 2 resources are in sync
- This indicates configuration drift that may need attention

### 5. Server-Side Diff with Namespace Filter

**User Query:**
> "Check the server-side diff for my-app in the argocd namespace"

**Tool Call:**
```json
{
  "tool": "server_side_diff",
  "arguments": {
    "app_name": "my-app",
    "app_namespace": "argocd",
    "project": "default"
  }
}
```

**Expected Response:**
```
Server-Side Diff for application 'my-app'
Total resources: 1, Modified: 0, In sync: 1

Modified Resources:
  (none)

In Sync Resources:
1. app-config (ConfigMap) in namespace 'default'
```

**Interpretation:**
- All resources are in sync with the target state
- No configuration drift detected
- The application is ready for production

### 6. Validate Changes Before Sync

**User Query:**
> "Before I sync the backend-api application, can you check what changes will be applied?"

**Tool Call:**
```json
{
  "tool": "server_side_diff",
  "arguments": {
    "app_name": "backend-api"
  }
}
```

**Expected Response:**
```
Server-Side Diff for application 'backend-api'
Total resources: 5, Modified: 2, In sync: 3

Modified Resources:
1. api-deployment (Deployment) in namespace 'backend-prod'
   Status: Resource has differences between live and target state
2. api-config (ConfigMap) in namespace 'backend-prod'
   Status: Resource has differences between live and target state

In Sync Resources:
1. api-service (Service) in namespace 'backend-prod'
2. api-ingress (Ingress) in namespace 'backend-prod'
3. api-secret (Secret) in namespace 'backend-prod'
```

**Assistant Response:**
> "Before syncing, I can see that 2 out of 5 resources will be modified:
> - **api-deployment**: This will update the Deployment configuration
> - **api-config**: This will update the ConfigMap
>
> The other 3 resources (Service, Ingress, and Secret) are already in sync and won't be changed. Would you like to proceed with the sync?"

## Advanced Use Cases

### Continuous Monitoring Workflow

**Scenario:** Monitor applications for drift every hour

```json
{
  "workflow": [
    {
      "step": 1,
      "tool": "list_application_names",
      "purpose": "Get all application names"
    },
    {
      "step": 2,
      "tool": "server_side_diff",
      "arguments": {
        "app_name": "<each_app_from_step_1>"
      },
      "purpose": "Check each application for drift"
    }
  ]
}
```

### Troubleshooting Out-of-Sync Applications

**User Query:**
> "Why is my helm-app showing as OutOfSync?"

**Workflow:**
1. First, list the application details:
```json
{
  "tool": "list_applications",
  "arguments": {
    "name": "helm-app"
  }
}
```

2. Then, check the detailed diff:
```json
{
  "tool": "server_side_diff",
  "arguments": {
    "app_name": "helm-app"
  }
}
```

**Assistant Analysis:**
> "I've checked the helm-app application. The server-side diff shows that:
> - 2 resources have differences: the Deployment and ConfigMap
> - The live Deployment has 2 replicas, but the target has 3 replicas
> - The ConfigMap has an outdated configuration value
>
> These differences are causing the OutOfSync status. Would you like me to help you sync the application?"

## Error Handling Examples

### Application Not Found

**Tool Call:**
```json
{
  "tool": "server_side_diff",
  "arguments": {
    "app_name": "nonexistent-app"
  }
}
```

**Expected Error:**
```
Error: ArgoCD API error (404): Application 'nonexistent-app' not found
```

### Authentication Error

**Expected Error:**
```
Error: ArgoCD API error (401): Invalid authentication token
```

**Solution:** Verify your ARGOCD_ACCESS_TOKEN environment variable

### Network Timeout

**Expected Error:**
```
Error: Failed to send request to ArgoCD API
```

**Solution:** Check network connectivity to the ArgoCD server

## Performance Tips

1. **Use `list_application_names` for quick lookups** - It's ~95% smaller than full application details
2. **Filter applications early** - Use `projects`, `selector`, or `repo` filters to reduce response size
3. **Server-side diff is optimized** - Only essential diff information is returned to save context window

## Integration with CI/CD

### Pre-Sync Validation

Add to your CI/CD pipeline:

```yaml
steps:
  - name: Check ArgoCD Diff
    run: |
      # Use MCP server to check what changes will be applied
      mcp-client call server_side_diff --app-name $APP_NAME

  - name: Review Changes
    run: |
      # Manual approval or automated validation
      # based on the diff results
```

## Limitations

1. **Server-Side Diff** is a beta feature (ArgoCD v2.10+)
2. Target manifests parameter is optional but may be needed for advanced scenarios
3. Large applications may take longer to calculate diffs

## Support

For issues or questions:
- Check the [README.md](README.md) for configuration details
- Review the [ArgoCD documentation](https://argo-cd.readthedocs.io/)
- Report issues at: https://github.com/YOUR_REPO/issues
