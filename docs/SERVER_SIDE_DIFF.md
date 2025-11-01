# Server-Side Diff Implementation Documentation

## Overview

The `server_side_diff` tool implements ArgoCD's Server-Side Diff API endpoint, which performs diff calculation using Kubernetes Server-Side Apply in dry-run mode. This provides more accurate diff results by involving admission controllers and validation webhooks in the calculation.

## Architecture

### Components

1. **Models** (`src/models.rs`)
   - `ResourceDiff`: Represents the diff between live and target resource states
   - `ApplicationServerSideDiffResponse`: API response containing diff results
   - `ServerSideDiffSummary`: Optimized summary for MCP tool output

2. **Client** (`src/argocd_client.rs`)
   - `server_side_diff()`: Optimized method returning summaries
   - `server_side_diff_full()`: Full response method for testing/debugging

3. **MCP Tool** (`src/tools.rs`)
   - `server_side_diff` tool with `ServerSideDiffArgs`
   - Formats output for human readability and structured data

4. **Tests** (`tests/integration_test.rs`)
   - Comprehensive test suite with mock ArgoCD server
   - Tests for success, error, and edge cases

## API Details

### Endpoint

```
GET /api/v1/applications/{appName}/server-side-diff
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `appName` | string | Yes | Application name (path parameter) |
| `appNamespace` | string | No | Application's namespace |
| `project` | string | No | Project identifier |
| `targetManifests` | array[string] | No | Target manifests for comparison |

### Response Schema

```rust
pub struct ApplicationServerSideDiffResponse {
    pub items: Vec<ResourceDiff>,
    pub modified: bool,
}

pub struct ResourceDiff {
    pub group: Option<String>,
    pub kind: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub live_state: Option<String>,
    pub target_state: Option<String>,
    pub normalized_live_state: Option<String>,
    pub predicted_live_state: Option<String>,
    pub modified: Option<bool>,
    pub hook: Option<bool>,
}
```

## Optimization Strategy

### Context Window Efficiency

The implementation uses `ServerSideDiffSummary` to reduce response size:

```rust
pub struct ServerSideDiffSummary {
    pub resource_name: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub modified: bool,
    pub diff_summary: Option<String>,
}
```

**Size Reduction:** ~70% smaller than full `ResourceDiff` objects

### What's Excluded in Summaries

- Full YAML states (`live_state`, `target_state`)
- Normalized states (`normalized_live_state`, `predicted_live_state`)
- Group information (can be inferred from kind)
- Hook status (rarely needed for basic diff analysis)

### When to Use Full Response

Use `server_side_diff_full()` when:
- Debugging specific resource differences
- Need to compare actual YAML content
- Building diff visualization tools
- Investigating complex sync issues

## Implementation Details

### Error Handling

The implementation handles:
1. **Authentication errors (401)**: Invalid or expired token
2. **Not found errors (404)**: Application doesn't exist
3. **Server errors (500)**: ArgoCD internal errors
4. **Network timeouts**: Connection issues
5. **Invalid JSON**: Malformed responses

Error messages are formatted for clarity:
```rust
"ArgoCD API error (404): Application 'nonexistent' not found"
```

### URL Encoding

Application names are properly URL-encoded:
```rust
format!("{}/api/v1/applications/{}/server-side-diff",
    self.base_url,
    urlencoding::encode(&app_name)
)
```

This handles special characters in application names.

### Query Parameter Handling

Query parameters are built dynamically:
```rust
if let Some(ns) = app_namespace {
    params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
}
```

Only non-None parameters are included in the request.

## Testing Strategy

### Mock Server Setup

Uses `wiremock` to simulate ArgoCD API:

```rust
let mock_server = MockServer::start().await;

Mock::given(method("GET"))
    .and(path("/api/v1/applications/guestbook/server-side-diff"))
    .and(header("Authorization", "Bearer test-token"))
    .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
    .mount(&mock_server)
    .await;
```

### Test Coverage

1. **Success Cases**
   - Basic diff with modified resources
   - Diff with query parameters
   - No changes scenario
   - Empty response

2. **Error Cases**
   - Authentication failure
   - Application not found
   - Server error
   - Network timeout

3. **Edge Cases**
   - Empty application list
   - All resources in sync
   - Mixed modified/in-sync resources

### Test Response Format

Mock responses follow ArgoCD API specification:

```json
{
  "modified": true,
  "items": [
    {
      "group": "apps",
      "kind": "Deployment",
      "namespace": "default",
      "name": "guestbook-ui",
      "liveState": "...",
      "targetState": "...",
      "normalizedLiveState": "...",
      "predictedLiveState": "...",
      "modified": true,
      "hook": false
    }
  ]
}
```

## Output Format

### Human-Readable Output

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

### JSON Output

Includes structured data for programmatic consumption:

```json
[
  {
    "resource_name": "guestbook-ui",
    "kind": "Deployment",
    "namespace": "default",
    "modified": true,
    "diff_summary": "Resource has differences between live and target state"
  }
]
```

## Performance Considerations

### Response Time

- Typical response time: < 500ms (depends on ArgoCD server)
- Timeout: 30 seconds (configurable in client)

### Memory Usage

- Summary objects: ~200 bytes per resource
- Full response: ~2-10KB per resource (varies with YAML size)

### Concurrent Requests

The client is fully async using Tokio:
```rust
pub async fn server_side_diff(&self, ...) -> Result<Vec<ServerSideDiffSummary>>
```

Multiple diff operations can run concurrently.

## Future Enhancements

### Potential Improvements

1. **Diff Analysis**
   - Parse YAML states to show specific field changes
   - Highlight important changes (replicas, images, etc.)
   - Categorize changes by severity

2. **Caching**
   - Cache diff results for short periods
   - Reduce API calls for repeated queries

3. **Filtering**
   - Filter by resource kind
   - Show only modified resources
   - Exclude specific namespaces

4. **Comparison Tools**
   - Side-by-side YAML comparison
   - Visual diff rendering
   - Change history tracking

## Security Considerations

1. **Authentication**
   - Bearer token passed via Authorization header
   - Token stored in environment variable
   - Never logged or exposed in responses

2. **Input Validation**
   - Application names are URL-encoded
   - Query parameters are validated
   - Error messages don't expose sensitive data

3. **HTTPS**
   - Always use HTTPS for production
   - Certificate validation enabled by default

## Troubleshooting

### Common Issues

1. **"Application not found"**
   - Verify application name spelling
   - Check application namespace parameter
   - Use `list_application_names` to verify existence

2. **"Unauthorized"**
   - Check ARGOCD_ACCESS_TOKEN is set
   - Verify token hasn't expired
   - Generate new token: `argocd account generate-token`

3. **"Timeout"**
   - Check network connectivity
   - Verify ArgoCD server is accessible
   - Consider increasing timeout in client configuration

4. **Empty Results**
   - Application may have no resources
   - Check application sync status
   - Verify application is deployed

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

This shows:
- API request URLs
- Response status codes
- Detailed error messages

## References

- [ArgoCD API Documentation](https://cd.apps.argoproj.io/swagger-ui)
- [Server-Side Diff Documentation](https://argo-cd.readthedocs.io/en/stable/user-guide/diff-strategies/)
- [Kubernetes Server-Side Apply](https://kubernetes.io/docs/reference/using-api/server-side-apply/)
- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)

## Version History

- **v0.1.0** (2025-11-01): Initial implementation
  - Basic server-side diff functionality
  - Optimized response format
  - Comprehensive test suite
  - Documentation and examples
