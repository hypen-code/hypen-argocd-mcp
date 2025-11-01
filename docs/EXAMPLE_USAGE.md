# ArgoCD MCP Server - Usage Examples

## Quick Start

### 1. Set Environment Variables

```bash
export ARGOCD_BASE_URL="https://cd.apps.argoproj.io"
export ARGOCD_ACCESS_TOKEN="your-token-here"
```

### 2. Run the Server

```bash
cargo run --release
```

### 3. Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector cargo run --release
```

## Example Tool Calls

### List All Applications

```json
{
  "name": "list_applications",
  "arguments": {}
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
   ...
```

### Filter by Project

```json
{
  "name": "list_applications",
  "arguments": {
    "projects": ["default"]
  }
}
```

### Filter by Labels

```json
{
  "name": "list_applications",
  "arguments": {
    "selector": "env=production"
  }
}
```

### Filter by Repository

```json
{
  "name": "list_applications",
  "arguments": {
    "repo": "https://github.com/argoproj/argocd-example-apps"
  }
}
```

### Multiple Filters

```json
{
  "name": "list_applications",
  "arguments": {
    "projects": ["default", "platform"],
    "selector": "env=production",
    "app_namespace": "argocd"
  }
}
```

### List Application Names Only

**Use this for efficient name lookups and typo correction!**

```json
{
  "name": "list_application_names",
  "arguments": {}
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

**Benefits:**
- ~95% smaller response than full application details
- Perfect for verifying application existence
- Ideal for auto-correcting typos (e.g., "guestbok" → "guestbook")
- Minimal context window usage

### List Application Names with Filters

```json
{
  "name": "list_application_names",
  "arguments": {
    "projects": ["default"],
    "selector": "env=production"
  }
}
```

## Integration Examples

### Claude Desktop Configuration

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "argocd": {
      "command": "/Users/yourname/argocd-mcp-server/target/release/argocd-mcp-server",
      "env": {
        "ARGOCD_BASE_URL": "https://argocd.example.com",
        "ARGOCD_ACCESS_TOKEN": "your-token"
      }
    }
  }
}
```

### Example Conversation with Claude

**User:** "Show me all applications in the default project"

**Claude:** *Uses list_applications tool with projects filter*
```
I found 5 applications in the default project:

1. guestbook - Healthy and Synced
2. helm-guestbook - Degraded, requires attention
3. kustomize-app - Healthy and Synced
...
```

**User:** "Which applications are out of sync?"

**Claude:** *Parses previous response*
```
Based on the current status, the following application is out of sync:
- helm-guestbook (Status: OutOfSync)

Would you like me to check for more details?
```

**User:** "Check the status of guestbok"

**Claude:** *Uses list_application_names first to find the correct name*
```
I noticed you may have meant "guestbook" (not "guestbok"). Let me check that application for you.

*Uses list_applications with name filter*

The guestbook application is:
- Health Status: Healthy
- Sync Status: Synced
- Project: default
- Repository: https://github.com/argoproj/argocd-example-apps
```

**Explanation:** The model first calls `list_application_names` to get all valid names, notices the typo, auto-corrects it to "guestbook", then retrieves the full details.

### Using with Custom Scripts

```bash
#!/bin/bash

# Create a named pipe for communication
mkfifo /tmp/mcp_pipe

# Start the MCP server
ARGOCD_BASE_URL="https://argocd.example.com" \
ARGOCD_ACCESS_TOKEN="your-token" \
./target/release/argocd-mcp-server < /tmp/mcp_pipe > /tmp/mcp_output &

# Send MCP request
cat << 'EOF' > /tmp/mcp_pipe
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_applications",
    "arguments": {}
  }
}
EOF

# Read response
cat /tmp/mcp_output
```

## Response Format

### Human-Readable Format
The tool returns formatted text optimized for readability:
```
Found 2 application(s):

1. app-name
   Project: default
   Repository: https://github.com/...
   Sync Status: Synced
   Health Status: Healthy
```

### JSON Format
Additionally includes structured JSON data:
```json
{
  "applications": [
    {
      "name": "app-name",
      "namespace": "argocd",
      "project": "default",
      "repo_url": "https://github.com/...",
      "target_revision": "HEAD",
      "destination_server": "https://kubernetes.default.svc",
      "destination_namespace": "default",
      "sync_status": "Synced",
      "health_status": "Healthy",
      "auto_sync": true
    }
  ],
  "count": 1
}
```

## Common Use Cases

### 1. Monitor Application Health

```json
{
  "name": "list_applications",
  "arguments": {}
}
```
Then check the `health_status` field for each application.

### 2. Find Out of Sync Applications

Filter results by parsing the response and looking for `sync_status != "Synced"`.

### 3. List Production Applications

```json
{
  "name": "list_applications",
  "arguments": {
    "selector": "env=production"
  }
}
```

### 4. Check Specific Project Status

```json
{
  "name": "list_applications",
  "arguments": {
    "projects": ["my-project"]
  }
}
```

### 5. Find Applications Using Specific Repository

```json
{
  "name": "list_applications",
  "arguments": {
    "repo": "https://github.com/myorg/myrepo"
  }
}
```

## Error Handling

### Authentication Error
```
ArgoCD API error (401): Invalid authentication token
```

### Network Error
```
Failed to list applications: error sending request
```

### Server Error
```
ArgoCD API error (500): Internal Server Error
```

## Performance Tips

1. **Use Filters**: Always filter when possible to reduce response size
2. **Cache Results**: The response includes all necessary data for caching
3. **Parallel Requests**: The server handles concurrent requests efficiently
4. **Check Logs**: Use `RUST_LOG=debug` for troubleshooting

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug ./target/release/argocd-mcp-server
```

### Test Connectivity
```bash
curl -H "Authorization: Bearer $ARGOCD_ACCESS_TOKEN" \
  "$ARGOCD_BASE_URL/api/v1/applications"
```

### Verify Token
```bash
argocd account get-user-info
```
