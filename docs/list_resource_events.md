# ListResourceEvents Implementation

## Overview

This document describes the implementation of the `list_resource_events` MCP tool for the ArgoCD MCP server.

## API Endpoint

**URL:** `GET /api/v1/applications/{name}/events`

**Swagger Reference:** https://cd.apps.argoproj.io/swagger-ui#tag/ApplicationService/operation/ApplicationService_ListResourceEvents

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Application name (path parameter) |
| `resourceNamespace` | string | No | Filter events by resource namespace |
| `resourceName` | string | No | Filter events by resource name |
| `resourceUID` | string | No | Filter events by resource UID |
| `appNamespace` | string | No | Application's namespace |
| `project` | string | No | Project identifier |

## Response Format

The endpoint returns a Kubernetes `v1.EventList` containing:

```json
{
  "metadata": {
    "resourceVersion": "string"
  },
  "items": [
    {
      "metadata": {
        "name": "string",
        "namespace": "string",
        "uid": "string"
      },
      "involvedObject": {
        "kind": "string",
        "namespace": "string",
        "name": "string",
        "uid": "string"
      },
      "reason": "string",
      "message": "string",
      "source": {
        "component": "string",
        "host": "string"
      },
      "firstTimestamp": "string",
      "lastTimestamp": "string",
      "count": 0,
      "type": "Normal|Warning"
    }
  ]
}
```

## Implementation Details

### Files Modified

1. **src/models.rs**
   - Added `EventList`, `Event`, `EventMetadata`, `ObjectReference`, `EventSource`, `EventSeries` models
   - Added `EventListSummary` and `EventSummary` for optimized responses
   - Implemented conversion traits for context efficiency

2. **src/argocd_client.rs**
   - Added `list_resource_events()` method returning optimized `EventListSummary`
   - Added `list_resource_events_full()` method returning full `EventList`
   - Implemented proper URL encoding and error handling

3. **src/tools.rs**
   - Added `ListResourceEventsArgs` struct for parameter validation
   - Implemented `list_resource_events` tool with #[tool] macro
   - Added formatted output with event grouping and summaries
   - Updated server instructions to include the new tool

4. **tests/integration_test.rs**
   - Added 11 comprehensive integration tests
   - Created mock response function `create_mock_resource_events_response()`
   - Tested all query parameters and error scenarios

### Key Features

1. **Context Optimization**
   - Events are aggregated by type and reason
   - Only the first 20 events are shown in detail to prevent context overflow
   - Summary statistics provided for quick insights

2. **Error Handling**
   - Comprehensive error handling for 401, 403, 404, 500 responses
   - Malformed JSON detection
   - Network timeout handling

3. **URL Encoding**
   - Proper encoding of application names with special characters
   - Query parameter encoding for all optional filters

4. **Response Formatting**
   - Human-readable output with clear event categorization
   - JSON output included for structured consumption
   - Event counts and summaries for quick analysis

## Test Coverage

### Unit Tests (5 total)
- Client creation validation
- Empty URL handling
- Empty token handling
- Handler creation
- Handler initialization

### Integration Tests (11 for list_resource_events)
1. `test_list_resource_events` - Basic functionality with multiple event types
2. `test_list_resource_events_with_filters` - Resource-specific filtering
3. `test_list_resource_events_empty` - Empty event list handling
4. `test_list_resource_events_authentication_error` - 401 error handling
5. `test_list_resource_events_not_found` - 404 error handling
6. `test_list_resource_events_server_error` - 500 error handling
7. `test_list_resource_events_full` - Full response details
8. `test_list_resource_events_all_parameters` - All query parameters
9. `test_list_resource_events_with_special_characters` - URL encoding
10. `test_list_resource_events_malformed_response` - Invalid JSON handling
11. `test_list_resource_events_forbidden` - 403 error handling

**Total Tests:** 65 (including existing tests)
**All Passing:** ✓

## Usage Examples

### List all events for an application
```json
{
  "application_name": "guestbook"
}
```

### Filter events for a specific deployment
```json
{
  "application_name": "guestbook",
  "resource_namespace": "default",
  "resource_name": "guestbook-ui",
  "resource_kind": "Deployment"
}
```

### Filter by resource UID
```json
{
  "application_name": "guestbook",
  "resource_uid": "abc-123-def-456"
}
```

## Performance Considerations

1. **Response Size**: Events are limited to 20 in the detailed output to prevent overwhelming the LLM context window
2. **Aggregation**: Events are pre-aggregated by type and reason for quick insights
3. **Filtering**: Server-side filtering reduces network traffic and processing

## Future Enhancements

Potential improvements:
- Event streaming for real-time monitoring
- Custom event retention policies
- Event correlation and pattern detection
- Export to external monitoring systems
- Advanced filtering by time range

## References

- [ArgoCD API Documentation](https://cd.apps.argoproj.io/swagger-ui)
- [Kubernetes Events API](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.28/#event-v1-core)
- [Model Context Protocol](https://github.com/modelcontextprotocol)
