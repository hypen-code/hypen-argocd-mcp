use anyhow::Result;
use serde_json::json;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

use argocd_mcp_server::argocd_client::ArgocdClient;

/// Mock response for resource events based on ArgoCD swagger specification
fn create_mock_events_response() -> serde_json::Value {
    json!({
        "metadata": {
            "resourceVersion": "12345"
        },
        "items": [
            {
                "metadata": {
                    "name": "deployment-event.17a1b2c3d4e5f6",
                    "namespace": "default",
                    "uid": "abc123-def456",
                    "resourceVersion": "100",
                    "creationTimestamp": "2025-01-01T10:00:00Z"
                },
                "involvedObject": {
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "guestbook-ui",
                    "uid": "deployment-uid-123",
                    "apiVersion": "apps/v1"
                },
                "reason": "ScalingReplicaSet",
                "message": "Scaled up replica set guestbook-ui-xyz to 3",
                "source": {
                    "component": "deployment-controller"
                },
                "firstTimestamp": "2025-01-01T10:00:00Z",
                "lastTimestamp": "2025-01-01T10:00:00Z",
                "count": 1,
                "type": "Normal"
            },
            {
                "metadata": {
                    "name": "pod-event.17a1b2c3d4e5f7",
                    "namespace": "default"
                },
                "involvedObject": {
                    "kind": "Pod",
                    "namespace": "default",
                    "name": "guestbook-ui-xyz-abc",
                    "uid": "pod-uid-456"
                },
                "reason": "Pulled",
                "message": "Container image already present on machine",
                "source": {
                    "component": "kubelet",
                    "host": "node-1"
                },
                "firstTimestamp": "2025-01-01T10:00:05Z",
                "lastTimestamp": "2025-01-01T10:00:05Z",
                "count": 1,
                "type": "Normal"
            },
            {
                "metadata": {
                    "name": "pod-event.17a1b2c3d4e5f8",
                    "namespace": "default"
                },
                "involvedObject": {
                    "kind": "Pod",
                    "namespace": "default",
                    "name": "guestbook-ui-xyz-def",
                    "uid": "pod-uid-789"
                },
                "reason": "FailedScheduling",
                "message": "0/3 nodes are available: insufficient memory",
                "source": {
                    "component": "default-scheduler"
                },
                "firstTimestamp": "2025-01-01T10:00:10Z",
                "lastTimestamp": "2025-01-01T10:01:00Z",
                "count": 5,
                "type": "Warning"
            }
        ]
    })
}

#[tokio::test]
async fn test_list_resource_events_basic() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "guestbook";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_events_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_events, 3);
    assert_eq!(summary.events_by_type.get("Normal"), Some(&2));
    assert_eq!(summary.events_by_type.get("Warning"), Some(&1));
    assert_eq!(summary.events_by_reason.get("ScalingReplicaSet"), Some(&1));
    assert_eq!(summary.events_by_reason.get("Pulled"), Some(&1));
    assert_eq!(summary.events_by_reason.get("FailedScheduling"), Some(&1));

    // Verify event details
    assert_eq!(summary.events[0].reason, Some("ScalingReplicaSet".to_string()));
    assert_eq!(summary.events[0].event_type, Some("Normal".to_string()));
    assert_eq!(summary.events[0].involved_object_kind, Some("Deployment".to_string()));
    assert_eq!(summary.events[0].count, Some(1));

    assert_eq!(summary.events[2].event_type, Some("Warning".to_string()));
    assert_eq!(summary.events[2].count, Some(5));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_no_events() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "no-events-app";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_events, 0);
    assert!(summary.events.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_malformed_json() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "malformed-app";

    // Test with completely invalid JSON
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Should return empty list instead of error
    assert_eq!(summary.total_events, 0);
    assert!(summary.events.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_missing_items_field() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "missing-items-app";

    // Test with response that has no items field
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "resourceVersion": "123"
            }
            // No items field
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Should handle gracefully and return empty list
    assert_eq!(summary.total_events, 0);

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_null_items() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "null-items-app";

    // Test with null items field
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": null
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Should handle gracefully
    assert_eq!(summary.total_events, 0);

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_empty_response() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "empty-response-app";

    // Test with completely empty JSON object
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Should handle gracefully
    assert_eq!(summary.total_events, 0);

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_with_filters() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "filtered-app";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "event-1",
                        "namespace": "prod"
                    },
                    "involvedObject": {
                        "kind": "Pod",
                        "namespace": "prod",
                        "name": "my-pod"
                    },
                    "reason": "Started",
                    "message": "Started container",
                    "type": "Normal",
                    "firstTimestamp": "2025-01-01T10:00:00Z",
                    "lastTimestamp": "2025-01-01T10:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        Some("prod".to_string()), // resource_namespace
        Some("my-pod".to_string()), // resource_name
        None, // resource_uid
        None, // app_namespace
        None, // project
    ).await?;

    assert_eq!(summary.total_events, 1);
    assert_eq!(summary.events[0].involved_object_name, Some("my-pod".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "auth-app";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "nonexistent";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404") || error_msg.contains("Not Found"));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "error-app";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to retrieve events"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500") || error_msg.contains("Server Error"));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_full() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-app";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_events_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let event_list = client.list_resource_events_full(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(event_list.items.len(), 3);
    assert!(event_list.metadata.is_some());

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_partial_data() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "partial-app";

    // Test with events that have minimal fields
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/events", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [
                {
                    "metadata": {
                        "name": "minimal-event"
                    },
                    "reason": "Unknown"
                    // Missing many optional fields
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        app_name.to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_events, 1);
    assert_eq!(summary.events[0].reason, Some("Unknown".to_string()));

    Ok(())
}
