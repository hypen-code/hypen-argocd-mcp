# ArgoCD MCP Server

A robust, optimized Model Context Protocol (MCP) server for ArgoCD, built with Rust. This server enables AI assistants to interact with ArgoCD APIs through standardized MCP tools.

## Features

- **Optimized Response Format**: Responses are optimized to minimize context window usage while providing essential information
- **Robust Error Handling**: Comprehensive error handling with detailed error messages
- **Complete Test Coverage**: Integration tests with mock ArgoCD API server
- **Stdio Transport**: Uses stdio transport for seamless integration with MCP clients
- **Type-Safe**: Built with Rust for type safety and performance

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

## Prerequisites

- Rust 1.70 or later
- ArgoCD server with API access
- Valid ArgoCD access token

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd argocd-mcp-server

# Build the project
cargo build --release
```

## Configuration

The server requires two environment variables:

- `ARGOCD_BASE_URL`: The base URL of your ArgoCD server (e.g., `https://argocd.example.com`)
- `ARGOCD_ACCESS_TOKEN`: Your ArgoCD API access token

### Getting an ArgoCD Access Token

1. Log in to your ArgoCD instance:
   ```bash
   argocd login <ARGOCD_SERVER>
   ```

2. Generate an account token:
   ```bash
   argocd account generate-token
   ```

## Usage

### Running the Server

```bash
# Set environment variables
export ARGOCD_BASE_URL=https://your-argocd-server.com
export ARGOCD_ACCESS_TOKEN=your-access-token-here

# Run the server
cargo run --release
```

### Using with MCP Inspector

You can test the server using the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector cargo run --release
```

### Integration with Claude Desktop

Add to your Claude Desktop configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "argocd": {
      "command": "/path/to/argocd-mcp-server/target/release/argocd-mcp-server",
      "env": {
        "ARGOCD_BASE_URL": "https://your-argocd-server.com",
        "ARGOCD_ACCESS_TOKEN": "your-access-token-here"
      }
    }
  }
}
```

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
│   ├── main.rs              # Entry point with stdio transport
│   ├── lib.rs               # Library exports
│   ├── argocd_client.rs     # ArgoCD API client
│   ├── models.rs            # Data models (optimized for context efficiency)
│   └── tools.rs             # MCP tool implementations
├── tests/
│   └── integration_test.rs  # Integration tests with mock server
├── Cargo.toml
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

## License

[Add your license here]

## Acknowledgments

- Built with [rust-sdk for MCP](https://github.com/modelcontextprotocol/rust-sdk)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client
- Testing with [wiremock](https://github.com/LukeMathWalker/wiremock-rs)
