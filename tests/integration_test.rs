use anyhow::Result;
use serde_json::json;
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// Import the modules we need to test
use argocd_mcp_server::argocd_client::ArgocdClient;

/// Mock response based on ArgoCD swagger specification
fn create_mock_application_list_response() -> serde_json::Value {
    json!({
        "metadata": {
            "resourceVersion": "12345"
        },
        "items": [
            {
                "metadata": {
                    "name": "guestbook",
                    "namespace": "argocd",
                    "labels": {
                        "env": "production",
                        "team": "platform"
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
                        "status": "Healthy"
                    },
                    "sync": {
                        "status": "Synced",
                        "revision": "abc123def456"
                    },
                    "summary": {
                        "externalURLs": ["https://guestbook.example.com"],
                        "images": ["gcr.io/heptio-images/ks-guestbook-demo:0.2"]
                    }
                }
            },
            {
                "metadata": {
                    "name": "helm-app",
                    "namespace": "argocd",
                    "labels": {
                        "env": "staging"
                    },
                    "creationTimestamp": "2025-01-02T10:00:00Z"
                },
                "spec": {
                    "project": "platform",
                    "source": {
                        "repoURL": "https://charts.example.com",
                        "chart": "nginx",
                        "targetRevision": "1.2.3"
                    },
                    "destination": {
                        "server": "https://kubernetes.default.svc",
                        "namespace": "staging"
                    }
                },
                "status": {
                    "health": {
                        "status": "Progressing",
                        "message": "Deployment is progressing"
                    },
                    "sync": {
                        "status": "OutOfSync"
                    }
                }
            },
            {
                "metadata": {
                    "name": "backend-api",
                    "namespace": "argocd",
                    "labels": {
                        "env": "production",
                        "app": "backend"
                    },
                    "creationTimestamp": "2025-01-03T10:00:00Z"
                },
                "spec": {
                    "project": "backend",
                    "source": {
                        "repoURL": "https://github.com/example/backend-api",
                        "path": "k8s/overlays/production",
                        "targetRevision": "v2.1.0"
                    },
                    "destination": {
                        "server": "https://prod-cluster.example.com",
                        "namespace": "backend-prod"
                    },
                    "syncPolicy": {
                        "automated": {
                            "prune": false,
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
                        "revision": "v2.1.0-xyz789"
                    }
                }
            }
        ]
    })
}

#[tokio::test]
async fn test_list_all_applications() -> Result<()> {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_application_list_response()))
        .mount(&mock_server)
        .await;

    // Create client
    let client = ArgocdClient::new(
        mock_server.uri(),
        "test-token".to_string(),
    )?;

    // Test list applications
    let summaries = client.list_applications(None, None, None, None, None).await?;

    // Verify results
    assert_eq!(summaries.len(), 3);

    // Verify first application
    assert_eq!(summaries[0].name, "guestbook");
    assert_eq!(summaries[0].namespace, Some("argocd".to_string()));
    assert_eq!(summaries[0].project, Some("default".to_string()));
    assert_eq!(summaries[0].repo_url, Some("https://github.com/argoproj/argocd-example-apps".to_string()));
    assert_eq!(summaries[0].sync_status, Some("Synced".to_string()));
    assert_eq!(summaries[0].health_status, Some("Healthy".to_string()));
    assert_eq!(summaries[0].auto_sync, Some(true));

    // Verify second application
    assert_eq!(summaries[1].name, "helm-app");
    assert_eq!(summaries[1].project, Some("platform".to_string()));
    assert_eq!(summaries[1].sync_status, Some("OutOfSync".to_string()));
    assert_eq!(summaries[1].health_status, Some("Progressing".to_string()));
    assert_eq!(summaries[1].auto_sync, None);

    // Verify third application
    assert_eq!(summaries[2].name, "backend-api");
    assert_eq!(summaries[2].destination_namespace, Some("backend-prod".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_list_applications_with_filters() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup mock for filtered request
    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("projects", "default"))
        .and(query_param("selector", "env=production"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "guestbook",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "project": "default",
                        "source": {
                            "repoURL": "https://github.com/argoproj/argocd-example-apps"
                        },
                        "destination": {
                            "namespace": "default"
                        }
                    },
                    "status": {
                        "sync": {
                            "status": "Synced"
                        },
                        "health": {
                            "status": "Healthy"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;

    let summaries = client.list_applications(
        None,
        Some(vec!["default".to_string()]),
        Some("env=production".to_string()),
        None,
        None,
    ).await?;

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "guestbook");

    Ok(())
}

#[tokio::test]
async fn test_empty_application_list() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(None, None, None, None, None).await?;

    assert_eq!(summaries.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.list_applications(None, None, None, None, None).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}

#[tokio::test]
async fn test_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Database connection failed"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_applications(None, None, None, None, None).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500") || error_msg.contains("Server Error"));

    Ok(())
}

#[tokio::test]
async fn test_network_timeout() -> Result<()> {
    // This will fail to connect, simulating network issues
    let client = ArgocdClient::new(
        "http://non-existent-server-12345.local".to_string(),
        "test-token".to_string(),
    )?;

    let result = client.list_applications(None, None, None, None, None).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_full_application_list() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_application_list_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let app_list = client.list_applications_full(None, None, None, None, None).await?;

    assert_eq!(app_list.items.len(), 3);
    assert!(app_list.metadata.is_some());

    // Verify metadata
    assert_eq!(
        app_list.metadata.unwrap().resource_version,
        Some("12345".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_list_application_names() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_application_list_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let names = client.list_application_names(None, None, None, None).await?;

    assert_eq!(names.len(), 3);
    assert_eq!(names[0], "guestbook");
    assert_eq!(names[1], "helm-app");
    assert_eq!(names[2], "backend-api");

    Ok(())
}

#[tokio::test]
async fn test_list_application_names_with_filters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("projects", "default"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "guestbook",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "project": "default",
                        "source": {
                            "repoURL": "https://github.com/argoproj/argocd-example-apps"
                        },
                        "destination": {}
                    },
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let names = client.list_application_names(
        Some(vec!["default".to_string()]),
        None,
        None,
        None,
    ).await?;

    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "guestbook");

    Ok(())
}

#[tokio::test]
async fn test_list_application_names_empty() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let names = client.list_application_names(None, None, None, None).await?;

    assert_eq!(names.len(), 0);

    Ok(())
}

/// Mock response for server-side diff based on ArgoCD swagger specification
fn create_mock_server_side_diff_response() -> serde_json::Value {
    json!({
        "modified": true,
        "items": [
            {
                "group": "apps",
                "kind": "Deployment",
                "namespace": "default",
                "name": "guestbook-ui",
                "liveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"guestbook-ui\"},\"spec\":{\"replicas\":2}}",
                "targetState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"guestbook-ui\"},\"spec\":{\"replicas\":3}}",
                "normalizedLiveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"guestbook-ui\"},\"spec\":{\"replicas\":2}}",
                "predictedLiveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"guestbook-ui\"},\"spec\":{\"replicas\":3}}",
                "modified": true,
                "hook": false
            },
            {
                "group": "",
                "kind": "Service",
                "namespace": "default",
                "name": "guestbook-ui",
                "liveState": "{\"apiVersion\":\"v1\",\"kind\":\"Service\",\"metadata\":{\"name\":\"guestbook-ui\"}}",
                "targetState": "{\"apiVersion\":\"v1\",\"kind\":\"Service\",\"metadata\":{\"name\":\"guestbook-ui\"}}",
                "normalizedLiveState": "{\"apiVersion\":\"v1\",\"kind\":\"Service\",\"metadata\":{\"name\":\"guestbook-ui\"}}",
                "predictedLiveState": "{\"apiVersion\":\"v1\",\"kind\":\"Service\",\"metadata\":{\"name\":\"guestbook-ui\"}}",
                "modified": false,
                "hook": false
            },
            {
                "group": "apps",
                "kind": "Deployment",
                "namespace": "default",
                "name": "redis-master",
                "liveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"redis-master\"},\"spec\":{\"replicas\":1}}",
                "targetState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"redis-master\"},\"spec\":{\"replicas\":1}}",
                "normalizedLiveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"redis-master\"},\"spec\":{\"replicas\":1}}",
                "predictedLiveState": "{\"apiVersion\":\"apps/v1\",\"kind\":\"Deployment\",\"metadata\":{\"name\":\"redis-master\"},\"spec\":{\"replicas\":1}}",
                "modified": false,
                "hook": false
            }
        ]
    })
}

#[tokio::test]
async fn test_server_side_diff() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Setup mock response for server-side diff
    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook/server-side-diff"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_server_side_diff_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.server_side_diff(
        "guestbook".to_string(),
        None,
        None,
        None,
    ).await?;

    // Verify results
    assert_eq!(summaries.len(), 3);

    // Verify first resource (modified)
    assert_eq!(summaries[0].resource_name, "guestbook-ui");
    assert_eq!(summaries[0].kind, "Deployment");
    assert_eq!(summaries[0].namespace, Some("default".to_string()));
    assert!(summaries[0].modified);
    assert!(summaries[0].diff_summary.is_some());

    // Verify second resource (not modified)
    assert_eq!(summaries[1].resource_name, "guestbook-ui");
    assert_eq!(summaries[1].kind, "Service");
    assert!(!summaries[1].modified);
    assert!(summaries[1].diff_summary.is_none());

    // Verify third resource (not modified)
    assert_eq!(summaries[2].resource_name, "redis-master");
    assert_eq!(summaries[2].kind, "Deployment");
    assert!(!summaries[2].modified);

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_with_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app/server-side-diff"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": false,
            "items": [
                {
                    "kind": "ConfigMap",
                    "name": "app-config",
                    "namespace": "default",
                    "modified": false
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.server_side_diff(
        "my-app".to_string(),
        Some("argocd".to_string()),
        Some("default".to_string()),
        None,
    ).await?;

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].resource_name, "app-config");
    assert!(!summaries[0].modified);

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_no_changes() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/synced-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": false,
            "items": [
                {
                    "kind": "Deployment",
                    "name": "app",
                    "namespace": "default",
                    "modified": false
                },
                {
                    "kind": "Service",
                    "name": "app",
                    "namespace": "default",
                    "modified": false
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.server_side_diff(
        "synced-app".to_string(),
        None,
        None,
        None,
    ).await?;

    assert_eq!(summaries.len(), 2);
    assert!(!summaries.iter().any(|s| s.modified));

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_empty_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/empty-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": false,
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.server_side_diff(
        "empty-app".to_string(),
        None,
        None,
        None,
    ).await?;

    assert_eq!(summaries.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.server_side_diff(
        "test-app".to_string(),
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
async fn test_server_side_diff_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent/server-side-diff"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application 'nonexistent' not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "nonexistent".to_string(),
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
async fn test_server_side_diff_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/error-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to calculate diff"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "error-app".to_string(),
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
async fn test_server_side_diff_full() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_server_side_diff_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let response = client.server_side_diff_full(
        "full-app".to_string(),
        None,
        None,
        None,
    ).await?;

    assert!(response.modified);
    assert_eq!(response.items.len(), 3);

    // Verify full details are present
    assert!(response.items[0].live_state.is_some());
    assert!(response.items[0].target_state.is_some());
    assert!(response.items[0].normalized_live_state.is_some());
    assert!(response.items[0].predicted_live_state.is_some());

    Ok(())
}

// ===== Additional Edge Case Tests =====

#[tokio::test]
async fn test_application_name_with_special_characters() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Test with application name containing special characters
    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app%2Fnamespace/server-side-diff"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": false,
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "my-app/namespace".to_string(),
        None,
        None,
        None,
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_list_applications_multiple_projects() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("projects", "project1"))
        .and(query_param("projects", "project2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "app1",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "project": "project1",
                        "source": {
                            "repoURL": "https://github.com/example/repo"
                        },
                        "destination": {}
                    },
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(
        None,
        Some(vec!["project1".to_string(), "project2".to_string()]),
        None,
        None,
        None,
    ).await?;

    assert_eq!(summaries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_with_target_manifests() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/server-side-diff"))
        .and(query_param("targetManifests", "manifest1"))
        .and(query_param("targetManifests", "manifest2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": true,
            "items": [
                {
                    "kind": "Deployment",
                    "name": "app",
                    "namespace": "default",
                    "modified": true
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "test-app".to_string(),
        None,
        None,
        Some(vec!["manifest1".to_string(), "manifest2".to_string()]),
    ).await;

    assert!(result.is_ok());
    let summaries = result.unwrap();
    assert_eq!(summaries.len(), 1);
    assert!(summaries[0].modified);

    Ok(())
}

#[tokio::test]
async fn test_malformed_json_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_applications(None, None, None, None, None).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));

    Ok(())
}

#[tokio::test]
async fn test_list_applications_with_name_filter() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("name", "specific-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "specific-app",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "project": "default",
                        "source": {
                            "repoURL": "https://github.com/example/repo"
                        },
                        "destination": {
                            "namespace": "default"
                        }
                    },
                    "status": {
                        "sync": {
                            "status": "Synced"
                        },
                        "health": {
                            "status": "Healthy"
                        }
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(
        Some("specific-app".to_string()),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].name, "specific-app");

    Ok(())
}

#[tokio::test]
async fn test_list_applications_with_repo_filter() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("repo", "https://github.com/example/repo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "repo-app",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "source": {
                            "repoURL": "https://github.com/example/repo"
                        },
                        "destination": {}
                    },
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(
        None,
        None,
        None,
        Some("https://github.com/example/repo".to_string()),
        None,
    ).await?;

    assert_eq!(summaries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_list_applications_with_app_namespace_filter() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("appNamespace", "custom-namespace"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "ns-app",
                        "namespace": "custom-namespace"
                    },
                    "spec": {
                        "source": {
                            "repoURL": "https://github.com/example/repo"
                        },
                        "destination": {}
                    },
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(
        None,
        None,
        None,
        None,
        Some("custom-namespace".to_string()),
    ).await?;

    assert_eq!(summaries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_list_applications_all_filters_combined() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("name", "my-app"))
        .and(query_param("projects", "default"))
        .and(query_param("selector", "env=prod"))
        .and(query_param("repo", "https://github.com/example/repo"))
        .and(query_param("appNamespace", "argocd"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "my-app",
                        "namespace": "argocd"
                    },
                    "spec": {
                        "project": "default",
                        "source": {
                            "repoURL": "https://github.com/example/repo"
                        },
                        "destination": {}
                    },
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summaries = client.list_applications(
        Some("my-app".to_string()),
        Some(vec!["default".to_string()]),
        Some("env=prod".to_string()),
        Some("https://github.com/example/repo".to_string()),
        Some("argocd".to_string()),
    ).await?;

    assert_eq!(summaries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_all_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-params-app/server-side-diff"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(query_param("targetManifests", "manifest1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "modified": true,
            "items": [
                {
                    "kind": "Deployment",
                    "name": "app",
                    "namespace": "default",
                    "modified": true
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "full-params-app".to_string(),
        Some("argocd".to_string()),
        Some("default".to_string()),
        Some(vec!["manifest1".to_string()]),
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_client_with_trailing_slash_in_url() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    // Create client with trailing slash - should be handled
    let url_with_slash = format!("{}/", mock_server.uri());
    let client = ArgocdClient::new(url_with_slash, "test-token".to_string())?;
    let result = client.list_applications(None, None, None, None, None).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_list_application_names_with_all_filters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .and(query_param("projects", "default"))
        .and(query_param("selector", "env=prod"))
        .and(query_param("repo", "https://github.com/example/repo"))
        .and(query_param("appNamespace", "argocd"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "filtered-app"
                    },
                    "spec": {},
                    "status": {}
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let names = client.list_application_names(
        Some(vec!["default".to_string()]),
        Some("env=prod".to_string()),
        Some("https://github.com/example/repo".to_string()),
        Some("argocd".to_string()),
    ).await?;

    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "filtered-app");

    Ok(())
}

#[tokio::test]
async fn test_error_response_with_empty_message() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": "",
            "message": ""
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_applications(None, None, None, None, None).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("400"));

    Ok(())
}

#[tokio::test]
async fn test_server_side_diff_403_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/forbidden-app/server-side-diff"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden",
            "message": "Permission denied"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.server_side_diff(
        "forbidden-app".to_string(),
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}

// ===== Resource Tree Tests =====

/// Mock response for resource tree based on ArgoCD swagger specification
fn create_mock_resource_tree_response() -> serde_json::Value {
    json!({
        "nodes": [
            {
                "version": "v1",
                "kind": "Deployment",
                "namespace": "default",
                "name": "guestbook-ui",
                "uid": "abc-123",
                "group": "apps",
                "resourceVersion": "12345",
                "images": ["gcr.io/heptio-images/ks-guestbook-demo:0.2"],
                "health": {
                    "status": "Healthy"
                },
                "createdAt": "2025-01-01T10:00:00Z"
            },
            {
                "version": "v1",
                "kind": "Service",
                "namespace": "default",
                "name": "guestbook-ui",
                "uid": "def-456",
                "resourceVersion": "12346",
                "health": {
                    "status": "Healthy"
                },
                "parentRefs": [
                    {
                        "group": "apps",
                        "version": "v1",
                        "kind": "Deployment",
                        "namespace": "default",
                        "name": "guestbook-ui",
                        "uid": "abc-123"
                    }
                ]
            },
            {
                "version": "v1",
                "kind": "Pod",
                "namespace": "default",
                "name": "guestbook-ui-12345",
                "uid": "ghi-789",
                "resourceVersion": "12347",
                "images": ["gcr.io/heptio-images/ks-guestbook-demo:0.2"],
                "health": {
                    "status": "Healthy"
                },
                "parentRefs": [
                    {
                        "group": "apps",
                        "version": "v1",
                        "kind": "ReplicaSet",
                        "namespace": "default",
                        "name": "guestbook-ui-abc",
                        "uid": "jkl-012"
                    }
                ]
            }
        ],
        "orphanedNodes": []
    })
}

#[tokio::test]
async fn test_resource_tree() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook/resource-tree"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_resource_tree_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.resource_tree(
        "guestbook".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Verify results
    assert_eq!(summary.total_nodes, 3);
    assert_eq!(summary.orphaned_nodes_count, 0);
    assert_eq!(*summary.nodes_by_kind.get("Deployment").unwrap(), 1);
    assert_eq!(*summary.nodes_by_kind.get("Service").unwrap(), 1);
    assert_eq!(*summary.nodes_by_kind.get("Pod").unwrap(), 1);
    assert_eq!(*summary.health_summary.get("Healthy").unwrap(), 3);
    assert_eq!(summary.sample_nodes.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_with_filters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app/resource-tree"))
        .and(query_param("namespace", "default"))
        .and(query_param("kind", "Deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "nodes": [
                {
                    "version": "v1",
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "my-deployment",
                    "uid": "abc-123",
                    "group": "apps",
                    "health": {
                        "status": "Healthy"
                    }
                }
            ],
            "orphanedNodes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.resource_tree(
        "my-app".to_string(),
        Some("default".to_string()),
        None,
        None,
        None,
        Some("Deployment".to_string()),
        None,
        None,
    ).await?;

    assert_eq!(summary.total_nodes, 1);
    assert_eq!(summary.sample_nodes[0].kind, "Deployment");

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_with_orphaned_nodes() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/resource-tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "nodes": [
                {
                    "version": "v1",
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "managed-deployment",
                    "uid": "abc-123",
                    "group": "apps",
                    "health": {
                        "status": "Healthy"
                    }
                }
            ],
            "orphanedNodes": [
                {
                    "version": "v1",
                    "kind": "ConfigMap",
                    "namespace": "default",
                    "name": "orphaned-config",
                    "uid": "def-456"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.resource_tree(
        "test-app".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_nodes, 1);
    assert_eq!(summary.orphaned_nodes_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_empty() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/empty-app/resource-tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "nodes": [],
            "orphanedNodes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.resource_tree(
        "empty-app".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_nodes, 0);
    assert_eq!(summary.orphaned_nodes_count, 0);
    assert_eq!(summary.sample_nodes.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/resource-tree"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.resource_tree(
        "test-app".to_string(),
        None,
        None,
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
async fn test_resource_tree_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent/resource-tree"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application 'nonexistent' not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.resource_tree(
        "nonexistent".to_string(),
        None,
        None,
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
async fn test_resource_tree_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/error-app/resource-tree"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to get resource tree"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.resource_tree(
        "error-app".to_string(),
        None,
        None,
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
async fn test_resource_tree_full() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-tree-app/resource-tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_resource_tree_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let tree = client.resource_tree_full(
        "full-tree-app".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(tree.nodes.len(), 3);
    assert!(tree.orphaned_nodes.is_some());
    assert_eq!(tree.orphaned_nodes.unwrap().len(), 0);

    // Verify detailed node information
    assert_eq!(tree.nodes[0].name, Some("guestbook-ui".to_string()));
    assert_eq!(tree.nodes[0].kind, Some("Deployment".to_string()));
    assert!(tree.nodes[0].images.is_some());
    assert!(tree.nodes[0].health.is_some());

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_all_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-params-app/resource-tree"))
        .and(query_param("namespace", "default"))
        .and(query_param("name", "my-deployment"))
        .and(query_param("version", "v1"))
        .and(query_param("group", "apps"))
        .and(query_param("kind", "Deployment"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "nodes": [
                {
                    "version": "v1",
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "my-deployment",
                    "uid": "abc-123",
                    "group": "apps",
                    "health": {
                        "status": "Healthy"
                    }
                }
            ],
            "orphanedNodes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.resource_tree(
        "full-params-app".to_string(),
        Some("default".to_string()),
        Some("my-deployment".to_string()),
        Some("v1".to_string()),
        Some("apps".to_string()),
        Some("Deployment".to_string()),
        Some("argocd".to_string()),
        Some("default".to_string()),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.total_nodes, 1);

    Ok(())
}

#[tokio::test]
async fn test_resource_tree_with_multiple_health_statuses() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/mixed-health-app/resource-tree"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "nodes": [
                {
                    "version": "v1",
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "healthy-deployment",
                    "uid": "abc-123",
                    "group": "apps",
                    "health": {
                        "status": "Healthy"
                    }
                },
                {
                    "version": "v1",
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "degraded-deployment",
                    "uid": "def-456",
                    "group": "apps",
                    "health": {
                        "status": "Degraded"
                    }
                },
                {
                    "version": "v1",
                    "kind": "Service",
                    "namespace": "default",
                    "name": "service-no-health",
                    "uid": "ghi-789"
                }
            ],
            "orphanedNodes": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.resource_tree(
        "mixed-health-app".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_nodes, 3);
    assert_eq!(*summary.health_summary.get("Healthy").unwrap(), 1);
    assert_eq!(*summary.health_summary.get("Degraded").unwrap(), 1);
    assert_eq!(*summary.health_summary.get("Unknown").unwrap(), 1);

    Ok(())
}

// ===== Get Application Tests =====

/// Mock response for get application based on ArgoCD swagger specification
fn create_mock_get_application_response() -> serde_json::Value {
    json!({
        "metadata": {
            "name": "guestbook",
            "namespace": "argocd",
            "labels": {
                "env": "production",
                "team": "platform"
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
                "status": "Healthy",
                "message": "All resources are healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "abc123def456"
            },
            "summary": {
                "externalURLs": ["https://guestbook.example.com"],
                "images": ["gcr.io/heptio-images/ks-guestbook-demo:0.2"]
            }
        }
    })
}

#[tokio::test]
async fn test_get_application() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_get_application_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let detail = client.get_application(
        "guestbook".to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    // Verify detailed information
    assert_eq!(detail.name, "guestbook");
    assert_eq!(detail.namespace, Some("argocd".to_string()));
    assert_eq!(detail.project, Some("default".to_string()));
    assert_eq!(detail.repo_url, Some("https://github.com/argoproj/argocd-example-apps".to_string()));
    assert_eq!(detail.path, Some("guestbook".to_string()));
    assert_eq!(detail.target_revision, Some("HEAD".to_string()));
    assert_eq!(detail.destination_server, Some("https://kubernetes.default.svc".to_string()));
    assert_eq!(detail.destination_namespace, Some("default".to_string()));
    assert_eq!(detail.sync_status, Some("Synced".to_string()));
    assert_eq!(detail.sync_revision, Some("abc123def456".to_string()));
    assert_eq!(detail.health_status, Some("Healthy".to_string()));
    assert_eq!(detail.health_message, Some("All resources are healthy".to_string()));
    assert_eq!(detail.auto_sync_enabled, Some(true));
    assert_eq!(detail.auto_sync_prune, Some(true));
    assert_eq!(detail.auto_sync_self_heal, Some(true));
    assert!(detail.labels.is_some());
    assert_eq!(detail.creation_timestamp, Some("2025-01-01T10:00:00Z".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_application_with_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "my-app",
                "namespace": "argocd"
            },
            "spec": {
                "project": "default",
                "source": {
                    "repoURL": "https://github.com/example/repo",
                    "path": "manifests"
                },
                "destination": {
                    "server": "https://kubernetes.default.svc",
                    "namespace": "production"
                }
            },
            "status": {
                "health": {
                    "status": "Healthy"
                },
                "sync": {
                    "status": "Synced"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let detail = client.get_application(
        "my-app".to_string(),
        Some("argocd".to_string()),
        Some("default".to_string()),
        None,
        None,
    ).await?;

    assert_eq!(detail.name, "my-app");
    assert_eq!(detail.namespace, Some("argocd".to_string()));
    assert_eq!(detail.project, Some("default".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_application_with_refresh() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/refresh-app"))
        .and(query_param("refresh", "hard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "refresh-app",
                "namespace": "argocd"
            },
            "spec": {
                "source": {
                    "repoURL": "https://github.com/example/repo"
                },
                "destination": {}
            },
            "status": {
                "sync": {
                    "status": "Synced"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "refresh-app".to_string(),
        None,
        None,
        Some("hard".to_string()),
        None,
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_application_helm_chart() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/helm-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "helm-app",
                "namespace": "argocd"
            },
            "spec": {
                "project": "platform",
                "source": {
                    "repoURL": "https://charts.example.com",
                    "chart": "nginx",
                    "targetRevision": "1.2.3"
                },
                "destination": {
                    "server": "https://kubernetes.default.svc",
                    "namespace": "staging"
                }
            },
            "status": {
                "health": {
                    "status": "Progressing",
                    "message": "Deployment is progressing"
                },
                "sync": {
                    "status": "OutOfSync"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let detail = client.get_application(
        "helm-app".to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(detail.name, "helm-app");
    assert_eq!(detail.chart, Some("nginx".to_string()));
    assert_eq!(detail.target_revision, Some("1.2.3".to_string()));
    assert_eq!(detail.health_status, Some("Progressing".to_string()));
    assert_eq!(detail.sync_status, Some("OutOfSync".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_application_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application 'nonexistent' not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "nonexistent".to_string(),
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
async fn test_get_application_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.get_application(
        "test-app".to_string(),
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
async fn test_get_application_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/forbidden-app"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden",
            "message": "Access denied to application 'forbidden-app'"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "forbidden-app".to_string(),
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}

#[tokio::test]
async fn test_get_application_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/error-app"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to retrieve application"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "error-app".to_string(),
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
async fn test_get_application_full() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_get_application_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let app = client.get_application_full(
        "full-app".to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert!(app.metadata.is_some());
    assert!(app.spec.is_some());
    assert!(app.status.is_some());

    let metadata = app.metadata.unwrap();
    assert_eq!(metadata.name, "guestbook");
    assert!(metadata.labels.is_some());

    let spec = app.spec.unwrap();
    assert!(spec.source.is_some());
    assert!(spec.destination.is_some());

    let status = app.status.unwrap();
    assert!(status.health.is_some());
    assert!(status.sync.is_some());

    Ok(())
}

#[tokio::test]
async fn test_get_application_with_all_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-params-app"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(query_param("refresh", "normal"))
        .and(query_param("resourceVersion", "12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "full-params-app",
                "namespace": "argocd"
            },
            "spec": {
                "project": "default",
                "source": {
                    "repoURL": "https://github.com/example/repo"
                },
                "destination": {}
            },
            "status": {
                "sync": {
                    "status": "Synced"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "full-params-app".to_string(),
        Some("argocd".to_string()),
        Some("default".to_string()),
        Some("normal".to_string()),
        Some("12345".to_string()),
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_application_with_special_characters_in_name() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app%2Fnamespace"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "my-app/namespace"
            },
            "spec": {
                "source": {
                    "repoURL": "https://github.com/example/repo"
                },
                "destination": {}
            },
            "status": {}
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "my-app/namespace".to_string(),
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_application_without_sync_policy() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/no-sync-policy-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {
                "name": "no-sync-policy-app",
                "namespace": "argocd"
            },
            "spec": {
                "project": "default",
                "source": {
                    "repoURL": "https://github.com/example/repo"
                },
                "destination": {
                    "namespace": "default"
                }
            },
            "status": {
                "sync": {
                    "status": "Synced"
                },
                "health": {
                    "status": "Healthy"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let detail = client.get_application(
        "no-sync-policy-app".to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(detail.name, "no-sync-policy-app");
    assert_eq!(detail.auto_sync_enabled, None);
    assert_eq!(detail.auto_sync_prune, None);
    assert_eq!(detail.auto_sync_self_heal, None);

    Ok(())
}

#[tokio::test]
async fn test_get_application_malformed_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/bad-response"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_application(
        "bad-response".to_string(),
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));

    Ok(())
}

// ===== ListResourceEvents Tests =====

/// Mock response for list resource events based on ArgoCD swagger specification
fn create_mock_resource_events_response() -> serde_json::Value {
    json!({
        "metadata": {
            "resourceVersion": "12345"
        },
        "items": [
            {
                "metadata": {
                    "name": "guestbook-ui.17a8b9c123456789",
                    "namespace": "default",
                    "uid": "event-123",
                    "resourceVersion": "12345",
                    "creationTimestamp": "2025-01-01T10:00:00Z"
                },
                "involvedObject": {
                    "kind": "Deployment",
                    "namespace": "default",
                    "name": "guestbook-ui",
                    "uid": "deployment-123",
                    "apiVersion": "apps/v1"
                },
                "reason": "ScalingReplicaSet",
                "message": "Scaled up replica set guestbook-ui-abc to 3",
                "source": {
                    "component": "deployment-controller"
                },
                "firstTimestamp": "2025-01-01T10:00:00Z",
                "lastTimestamp": "2025-01-01T10:05:00Z",
                "count": 5,
                "type": "Normal",
                "reportingComponent": "deployment-controller",
                "reportingInstance": "deployment-controller-xyz"
            },
            {
                "metadata": {
                    "name": "guestbook-ui.17a8b9c987654321",
                    "namespace": "default",
                    "uid": "event-456",
                    "resourceVersion": "12346",
                    "creationTimestamp": "2025-01-01T10:10:00Z"
                },
                "involvedObject": {
                    "kind": "Pod",
                    "namespace": "default",
                    "name": "guestbook-ui-abc-12345",
                    "uid": "pod-789",
                    "apiVersion": "v1"
                },
                "reason": "Started",
                "message": "Started container guestbook",
                "source": {
                    "component": "kubelet",
                    "host": "node-1"
                },
                "firstTimestamp": "2025-01-01T10:10:00Z",
                "lastTimestamp": "2025-01-01T10:10:00Z",
                "count": 1,
                "type": "Normal",
                "reportingComponent": "kubelet",
                "reportingInstance": "node-1"
            },
            {
                "metadata": {
                    "name": "guestbook-ui.17a8b9c111222333",
                    "namespace": "default",
                    "uid": "event-789",
                    "resourceVersion": "12347",
                    "creationTimestamp": "2025-01-01T10:15:00Z"
                },
                "involvedObject": {
                    "kind": "Pod",
                    "namespace": "default",
                    "name": "guestbook-ui-abc-67890",
                    "uid": "pod-101",
                    "apiVersion": "v1"
                },
                "reason": "FailedScheduling",
                "message": "0/5 nodes are available: insufficient cpu",
                "source": {
                    "component": "default-scheduler"
                },
                "firstTimestamp": "2025-01-01T10:15:00Z",
                "lastTimestamp": "2025-01-01T10:20:00Z",
                "count": 10,
                "type": "Warning",
                "reportingComponent": "default-scheduler",
                "reportingInstance": "scheduler-xyz"
            }
        ]
    })
}

#[tokio::test]
async fn test_list_resource_events() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook/events"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_resource_events_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        "guestbook".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Verify results
    assert_eq!(summary.total_events, 3);
    assert_eq!(*summary.events_by_type.get("Normal").unwrap(), 2);
    assert_eq!(*summary.events_by_type.get("Warning").unwrap(), 1);
    assert_eq!(*summary.events_by_reason.get("ScalingReplicaSet").unwrap(), 1);
    assert_eq!(*summary.events_by_reason.get("Started").unwrap(), 1);
    assert_eq!(*summary.events_by_reason.get("FailedScheduling").unwrap(), 1);

    // Verify event details
    assert_eq!(summary.events[0].event_type, Some("Normal".to_string()));
    assert_eq!(summary.events[0].reason, Some("ScalingReplicaSet".to_string()));
    assert_eq!(summary.events[0].involved_object_kind, Some("Deployment".to_string()));
    assert_eq!(summary.events[0].involved_object_name, Some("guestbook-ui".to_string()));
    assert_eq!(summary.events[0].count, Some(5));

    assert_eq!(summary.events[2].event_type, Some("Warning".to_string()));
    assert_eq!(summary.events[2].reason, Some("FailedScheduling".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_with_filters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app/events"))
        .and(query_param("resourceNamespace", "default"))
        .and(query_param("resourceName", "my-deployment"))
        .and(query_param("resourceUID", "deployment-uid-123"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "my-deployment.17a8b9c123",
                        "namespace": "default"
                    },
                    "involvedObject": {
                        "kind": "Deployment",
                        "namespace": "default",
                        "name": "my-deployment",
                        "uid": "deployment-uid-123"
                    },
                    "reason": "ScalingReplicaSet",
                    "message": "Scaled up replica set",
                    "source": {
                        "component": "deployment-controller"
                    },
                    "firstTimestamp": "2025-01-01T10:00:00Z",
                    "lastTimestamp": "2025-01-01T10:00:00Z",
                    "count": 1,
                    "type": "Normal"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        "my-app".to_string(),
        Some("default".to_string()),
        Some("my-deployment".to_string()),
        Some("deployment-uid-123".to_string()),
        None,
        None,
    ).await?;

    assert_eq!(summary.total_events, 1);
    assert_eq!(summary.events[0].involved_object_name, Some("my-deployment".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_empty() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/empty-app/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.list_resource_events(
        "empty-app".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_events, 0);
    assert_eq!(summary.events.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/events"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.list_resource_events(
        "test-app".to_string(),
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

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent/events"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application 'nonexistent' not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "nonexistent".to_string(),
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

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/error-app/events"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to retrieve events"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "error-app".to_string(),
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

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-events-app/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_resource_events_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let event_list = client.list_resource_events_full(
        "full-events-app".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(event_list.items.len(), 3);
    assert!(event_list.metadata.is_some());

    // Verify detailed event information
    assert!(event_list.items[0].metadata.is_some());
    assert!(event_list.items[0].involved_object.is_some());
    assert_eq!(event_list.items[0].reason, Some("ScalingReplicaSet".to_string()));
    assert!(event_list.items[0].message.is_some());
    assert!(event_list.items[0].source.is_some());

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_all_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/full-params-app/events"))
        .and(query_param("resourceNamespace", "default"))
        .and(query_param("resourceName", "my-deployment"))
        .and(query_param("resourceUID", "deployment-uid"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": [
                {
                    "metadata": {
                        "name": "event.123",
                        "namespace": "default"
                    },
                    "involvedObject": {
                        "kind": "Deployment",
                        "namespace": "default",
                        "name": "my-deployment",
                        "uid": "deployment-uid"
                    },
                    "reason": "ScalingReplicaSet",
                    "message": "Scaled up",
                    "type": "Normal",
                    "firstTimestamp": "2025-01-01T10:00:00Z",
                    "lastTimestamp": "2025-01-01T10:00:00Z",
                    "count": 1
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "full-params-app".to_string(),
        Some("default".to_string()),
        Some("my-deployment".to_string()),
        Some("deployment-uid".to_string()),
        Some("argocd".to_string()),
        Some("default".to_string()),
    ).await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.total_events, 1);

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_with_special_characters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app%2Fnamespace/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metadata": {},
            "items": []
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "my-app/namespace".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_malformed_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/bad-response/events"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "bad-response".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to parse"));

    Ok(())
}

#[tokio::test]
async fn test_list_resource_events_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/forbidden-app/events"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden",
            "message": "Permission denied"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.list_resource_events(
        "forbidden-app".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}

// ===== PodLogs Tests =====

/// Mock response for pod logs (NDJSON format)
fn create_mock_pod_logs_response() -> String {
    // ArgoCD returns logs as NDJSON (newline-delimited JSON)
    vec![
        r#"{"result":{"content":"2025-01-01T10:00:00Z INFO Starting application server","timeStampStr":"2025-01-01T10:00:00Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:01Z INFO Listening on port 8080","timeStampStr":"2025-01-01T10:00:01Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:05Z WARN Database connection pool running low","timeStampStr":"2025-01-01T10:00:05Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:10Z ERROR Failed to connect to database: connection timeout","timeStampStr":"2025-01-01T10:00:10Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:15Z ERROR Retrying connection...","timeStampStr":"2025-01-01T10:00:15Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:20Z INFO Successfully connected to database","timeStampStr":"2025-01-01T10:00:20Z","podName":"guestbook-ui-abc-123"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:25Z DEBUG Processing request #1","timeStampStr":"2025-01-01T10:00:25Z","podName":"guestbook-ui-abc-123"}}"#,
    ].join("\n")
}

fn create_mock_error_logs_response() -> String {
    vec![
        r#"{"result":{"content":"2025-01-01T10:00:00Z FATAL Panic: nil pointer dereference","timeStampStr":"2025-01-01T10:00:00Z","podName":"app-pod"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:01Z ERROR Exception in main loop","timeStampStr":"2025-01-01T10:00:01Z","podName":"app-pod"}}"#,
        r#"{"result":{"content":"2025-01-01T10:00:02Z ERROR Container failed to start","timeStampStr":"2025-01-01T10:00:02Z","podName":"app-pod"}}"#,
    ].join("\n")
}

#[tokio::test]
async fn test_pod_logs_basic() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook/logs"))
        .and(query_param("podName", "guestbook-ui-abc-123"))
        .and(query_param("follow", "false"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "guestbook".to_string(),
        None,
        Some("guestbook-ui-abc-123".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false, // don't filter errors
    ).await?;

    assert_eq!(summary.total_lines, 7);
    assert_eq!(summary.error_count, 2);
    assert_eq!(summary.warning_count, 1);
    assert!(!summary.filtered);

    // Check log levels
    assert_eq!(*summary.logs_by_level.get("INFO").unwrap(), 3);
    assert_eq!(*summary.logs_by_level.get("ERROR").unwrap(), 2);
    assert_eq!(*summary.logs_by_level.get("WARNING").unwrap(), 1);
    assert_eq!(*summary.logs_by_level.get("DEBUG").unwrap(), 1);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_error_filtering() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app/logs"))
        .and(query_param("podName", "app-pod"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "my-app".to_string(),
        None,
        Some("app-pod".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true, // filter errors only
    ).await?;

    // Should only have errors and warnings (3 total: 2 errors + 1 warning)
    assert_eq!(summary.total_lines, 3);
    assert_eq!(summary.error_count, 2);
    assert_eq!(summary.warning_count, 1);
    assert!(summary.filtered);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_tail_lines() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/test-app/logs"))
        .and(query_param("podName", "test-pod"))
        .and(query_param("tailLines", "50"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "test-app".to_string(),
        None,
        Some("test-pod".to_string()),
        None,
        None,
        Some(50),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.tail_lines, Some(50));

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_container() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("namespace", "default"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("container", "app-container"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        Some("default".to_string()),
        Some("pod-1".to_string()),
        Some("app-container".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.container, Some("app-container".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_since_seconds() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("sinceSeconds", "300"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        Some(300),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.total_lines, 7);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_previous() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("previous", "true"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        Some(true),
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.total_lines, 7);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_filter() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("filter", "ERROR"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_error_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        Some("ERROR".to_string()),
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.total_lines, 3);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_with_resource_name() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("kind", "Deployment"))
        .and(query_param("resourceName", "my-deployment"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_pod_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("Deployment".to_string()),
        None,
        Some("my-deployment".to_string()),
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.total_lines, 7);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_empty() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(""))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    assert_eq!(summary.total_lines, 0);
    assert_eq!(summary.error_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent/logs"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.pod_logs(
        "nonexistent".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("404") || error_msg.contains("Not Found"));

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to retrieve logs"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("500") || error_msg.contains("Server Error"));

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_all_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("namespace", "default"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("container", "app"))
        .and(query_param("sinceSeconds", "300"))
        .and(query_param("tailLines", "100"))
        .and(query_param("previous", "true"))
        .and(query_param("filter", "ERROR"))
        .and(query_param("kind", "Pod"))
        .and(query_param("resourceName", "pod-1"))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "default"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(create_mock_error_logs_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        Some("default".to_string()),
        Some("pod-1".to_string()),
        Some("app".to_string()),
        Some(300),
        Some(100),
        Some(true),
        Some("ERROR".to_string()),
        Some("Pod".to_string()),
        None,
        Some("pod-1".to_string()),
        Some("argocd".to_string()),
        Some("default".to_string()),
        false,
    ).await?;

    assert_eq!(summary.total_lines, 3);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_potential_issues_detection() -> Result<()> {
    let logs_with_issues = vec![
        r#"{"result":{"content":"Connection refused by remote server","timeStampStr":"2025-01-01T10:00:00Z"}}"#,
        r#"{"result":{"content":"Exception: timeout waiting for response","timeStampStr":"2025-01-01T10:00:01Z"}}"#,
        r#"{"result":{"content":"Unable to allocate memory","timeStampStr":"2025-01-01T10:00:02Z"}}"#,
        r#"{"result":{"content":"Panic: index out of bounds","timeStampStr":"2025-01-01T10:00:03Z"}}"#,
        r#"{"result":{"content":"Permission denied accessing file","timeStampStr":"2025-01-01T10:00:04Z"}}"#,
    ].join("\n");

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .and(query_param("podName", "pod-1"))
        .and(query_param("follow", "false"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(logs_with_issues))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await?;

    // All 5 logs should be detected as potential issues
    assert_eq!(summary.potential_issue_count, 5);

    // Now test with error filtering
    let summary_filtered = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        true, // filter errors
    ).await?;

    assert_eq!(summary_filtered.total_lines, 5);
    assert!(summary_filtered.filtered);

    Ok(())
}

#[tokio::test]
async fn test_pod_logs_forbidden() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/logs"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": "Forbidden",
            "message": "Access denied"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.pod_logs(
        "app".to_string(),
        None,
        Some("pod-1".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("403") || error_msg.contains("Forbidden"));

    Ok(())
}
