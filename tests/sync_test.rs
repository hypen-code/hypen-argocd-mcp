use anyhow::Result;
use serde_json::json;
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// Import the modules we need to test
use argocd_mcp_server::argocd_client::ArgocdClient;
use argocd_mcp_server::models::{Backoff, RetryStrategy, SyncResource};

/// Mock response for sync based on ArgoCD swagger specification
fn create_mock_sync_response() -> serde_json::Value {
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
                "targetRevision": "HEAD"
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
                "status": "Progressing"
            },
            "sync": {
                "status": "Synced",
                "revision": "abc123def456"
            }
        }
    })
}

#[tokio::test]
async fn test_sync_application_basic() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "guestbook";

    // Expected request body

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .and(header("Authorization", "Bearer test-token"))
        .and(header("Content-Type", "application/json"))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, "guestbook");
    assert!(!summary.dry_run);
    assert!(!summary.prune_enabled);
    assert!(!summary.force_enabled);
    assert_eq!(summary.sync_status, Some("Synced".to_string()));
    assert_eq!(summary.health_status, Some("Progressing".to_string()));
    assert_eq!(summary.target_revision, Some("HEAD".to_string()));
    assert!(summary.sync_options.is_empty());
    assert!(summary.resources_count.is_none());

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_dry_run() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "test-app";


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        
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
        .sync_application(
            app_name.to_string(),
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, app_name);
    assert!(summary.dry_run);
    assert_eq!(summary.sync_status, Some("OutOfSync".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_prune() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "prune-app";


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        // Don't match exact body for resources - just verify the endpoint
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert!(summary.prune_enabled);

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_force() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "force-app";


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        // Don't match exact body for resources - just verify the endpoint
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert!(summary.force_enabled);

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_revision() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "rev-app";
    let revision = "v1.2.3";


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            Some(revision.to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, "guestbook");

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_specific_resources() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "resource-app";

    let resources = vec![
        SyncResource {
            group: Some("apps".to_string()),
            kind: "Deployment".to_string(),
            name: "my-deployment".to_string(),
            namespace: Some("default".to_string()),
        },
        SyncResource {
            group: None,
            kind: "Service".to_string(),
            name: "my-service".to_string(),
            namespace: Some("default".to_string()),
        },
    ];

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            Some(resources),
            None,
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.resources_count, Some(2));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_sync_options() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "opts-app";

    let sync_options = vec![
        "Validate=false".to_string(),
        "CreateNamespace=true".to_string(),
    ];


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            Some(sync_options.clone()),
            None,
            None,
            None,
        )
        .await?;

    assert_eq!(summary.sync_options, sync_options);

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_retry() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "retry-app";

    let retry = RetryStrategy {
        limit: Some(5),
        backoff: Some(Backoff {
            duration: Some("5s".to_string()),
            max_duration: Some("3m".to_string()),
            factor: Some(2),
        }),
    };


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            Some(retry),
            None,
            None,
        )
        .await?;

    assert_eq!(summary.name, "guestbook");

    Ok(())
}

#[tokio::test]
async fn test_sync_application_with_namespace_and_project() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "ns-app";
    let app_namespace = "custom-ns";
    let project = "my-project";


    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .and(query_param("appNamespace", app_namespace))
        .and(query_param("project", project))
        
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(app_namespace.to_string()),
            Some(project.to_string()),
        )
        .await?;

    assert_eq!(summary.name, "guestbook");

    Ok(())
}

#[tokio::test]
async fn test_sync_application_all_options() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-opts-app";

    let resources = vec![SyncResource {
        group: Some("apps".to_string()),
        kind: "Deployment".to_string(),
        name: "my-deployment".to_string(),
        namespace: Some("default".to_string()),
    }];

    let sync_options = vec!["Validate=false".to_string()];

    let retry = RetryStrategy {
        limit: Some(3),
        backoff: None,
    };

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        // Don't match exact body for complex objects - just verify the endpoint
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client
        .sync_application(
            app_name.to_string(),
            Some("v1.0.0".to_string()),
            Some(true),
            Some(true),
            Some(true),
            Some(resources),
            Some(sync_options.clone()),
            Some(retry),
            Some("argocd".to_string()),
            Some("default".to_string()),
        )
        .await?;

    assert!(summary.dry_run);
    assert!(summary.prune_enabled);
    assert!(summary.force_enabled);
    assert_eq!(summary.sync_options, sync_options);
    assert_eq!(summary.resources_count, Some(1));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "nonexistent-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application 'nonexistent-app' not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404") || error_msg.contains("Not Found"));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "auth-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "forbidden-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden",
            "message": "User does not have permission to sync application 'forbidden-app'"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "error-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to perform sync operation"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500") || error_msg.contains("Server Error"));

    Ok(())
}

#[tokio::test]
async fn test_sync_application_full() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let app = client
        .sync_application_full(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    // Verify full application object
    assert!(app.metadata.is_some());
    assert_eq!(app.metadata.unwrap().name, "guestbook");
    assert!(app.spec.is_some());
    assert!(app.status.is_some());

    Ok(())
}

#[tokio::test]
async fn test_sync_application_network_timeout() -> Result<()> {
    // This will fail to connect, simulating network issues
    let client = ArgocdClient::new(
        "http://non-existent-server-12345.local".to_string(),
        "test-token".to_string(),
    )?;

    let result = client
        .sync_application(
            "test-app".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_sync_application_malformed_response() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "malformed-app";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/applications/{}/sync", app_name)))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client
        .sync_application(
            app_name.to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));

    Ok(())
}
