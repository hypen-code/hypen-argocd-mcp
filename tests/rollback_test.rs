use anyhow::Result;
use serde_json::json;
use wiremock::{
    matchers::{body_json, header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// Import the modules we need to test
use argocd_mcp_server::argocd_client::ArgocdClient;

/// Mock response for rollback based on ArgoCD swagger specification
fn create_mock_rollback_response() -> serde_json::Value {
    json!({
        "metadata": {
            "name": "guestbook",
            "namespace": "argocd",
            "labels": {
                "env": "production"
            },
            "creationTimestamp": "2025-01-01T10:00:00Z"
        },
        "spec": {
            "project": "default",
            "source": {
                "repoURL": "https://github.com/argoproj/argocd-example-apps",
                "path": "guestbook",
                "targetRevision": "abc123"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            },
            "syncPolicy": {
                "automated": {
                    "prune": true,
                    "selfHeal": true
                }
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "abc123def456"
            }
        }
    })
}

#[tokio::test]
async fn test_rollback_application_basic() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "guestbook";
    let history_id = 5i64;

    // Expected request body
    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": false,
        "prune": false,
        "appNamespace": null,
        "project": null
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(header("Authorization", "Bearer test-token"))
        .and(header("Content-Type", "application/json"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(
            app_name.to_string(),
            history_id,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, "guestbook");
    assert_eq!(summary.rolled_back_to_id, history_id);
    assert!(!summary.dry_run);
    assert!(!summary.prune_enabled);
    assert_eq!(summary.sync_status, Some("Synced".to_string()));
    assert_eq!(summary.health_status, Some("Healthy".to_string()));
    assert_eq!(summary.target_revision, Some("abc123".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_with_dry_run() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "test-app";
    let history_id = 3i64;

    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": true,
        "prune": false,
        "appNamespace": null,
        "project": null
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": app_name,
                "namespace": "argocd"
            },
            "spec": {
                "source": {
                    "repoURL": "https://github.com/example/repo",
                    "targetRevision": "v1.0.0"
                },
                "destination": {
                    "namespace": "default"
                }
            },
            "status": {
                "sync": {
                    "status": "OutOfSync",
                    "revision": "v2.0.0"
                },
                "health": {
                    "status": "Healthy"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(
            app_name.to_string(),
            history_id,
            Some(true),
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, app_name);
    assert_eq!(summary.rolled_back_to_id, history_id);
    assert!(summary.dry_run);
    assert_eq!(summary.sync_status, Some("OutOfSync".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_with_prune() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "prune-app";
    let history_id = 2i64;

    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": false,
        "prune": true,
        "appNamespace": null,
        "project": null
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(
            app_name.to_string(),
            history_id,
            None,
            Some(true),
            None,
            None,
        )
        .await?;

    assert!(summary.prune_enabled);

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_with_namespace_and_project() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "ns-app";
    let history_id = 4i64;
    let app_namespace = "custom-ns";
    let project = "my-project";

    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": false,
        "prune": false,
        "appNamespace": app_namespace,
        "project": project
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(query_param("appNamespace", app_namespace))
        .and(query_param("project", project))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(
            app_name.to_string(),
            history_id,
            None,
            None,
            Some(app_namespace.to_string()),
            Some(project.to_string()),
        )
        .await?;

    assert_eq!(summary.name, "guestbook");
    assert_eq!(summary.rolled_back_to_id, history_id);

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_all_options() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-options-app";
    let history_id = 10i64;

    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": true,
        "prune": true,
        "appNamespace": "argocd",
        "project": "default"
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(
            app_name.to_string(),
            history_id,
            Some(true),
            Some(true),
            Some("argocd".to_string()),
            Some("default".to_string()),
        )
        .await?;

    assert_eq!(summary.rolled_back_to_id, history_id);
    assert!(summary.dry_run);
    assert!(summary.prune_enabled);

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "nonexistent-app";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(json!({
                "error": "Not Found",
                "message": "Application 'nonexistent-app' not found"
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404") || error_msg.contains("Not Found"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_invalid_history_id() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "test-app";
    let history_id = 999i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(json!({
                "error": "Bad Request",
                "message": "History ID 999 not found for application 'test-app'"
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("400") || error_msg.contains("Bad Request"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "auth-app";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized",
                "message": "Invalid authentication token"
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "forbidden-app";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(json!({
                "error": "Forbidden",
                "message": "User does not have permission to rollback application 'forbidden-app'"
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "error-app";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(
            ResponseTemplate::new(500).set_body_json(json!({
                "error": "Internal Server Error",
                "message": "Failed to perform rollback operation"
            })),
        )
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500") || error_msg.contains("Server Error"));

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_full() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-app";
    let history_id = 7i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let app = client
        .rollback_application_full(app_name.to_string(), history_id, None, None, None, None)
        .await?;

    // Verify full application object
    assert!(app.metadata.is_some());
    assert_eq!(app.metadata.unwrap().name, "guestbook");
    assert!(app.spec.is_some());
    assert!(app.status.is_some());

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_with_special_characters() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "my-app/namespace";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path("/api/v1/applications/my-app%2Fnamespace/rollback"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_to_previous_version() -> Result<()> {
    // Test rolling back to history ID 0 or omitting it should rollback to previous version
    let mock_server = MockServer::start().await;
    let app_name = "prev-version-app";
    let history_id = 0i64; // 0 typically means "previous version"

    let expected_body = json!({
        "name": app_name,
        "id": history_id,
        "dryRun": false,
        "prune": false,
        "appNamespace": null,
        "project": null
    });

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .and(body_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_rollback_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await?;

    assert_eq!(summary.rolled_back_to_id, 0);

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_network_timeout() -> Result<()> {
    // This will fail to connect, simulating network issues
    let client = ArgocdClient::new(
        "http://non-existent-server-12345.local".to_string(),
        "test-token".to_string(),
    )?;

    let result = client
        .rollback_application("test-app".to_string(), 1, None, None, None, None)
        .await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_rollback_application_malformed_response() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "malformed-app";
    let history_id = 1i64;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/rollback", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .rollback_application(app_name.to_string(), history_id, None, None, None, None)
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));

    Ok(())
}
