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
    let url_with_slash = format!("{}/",

    mock_server.uri());
    let client = ArgocdClient::new(url_with_slash, "test-token".to_string())?;
    let summaries = client.list_applications(None, None, None, None, None).await?;

    assert_eq!(summaries.len(), 0);

    Ok(())
}

/// Mock response for revision metadata based on ArgoCD swagger specification
fn create_mock_revision_metadata_response() -> serde_json::Value {
    json!({
        "author": "John Doe <john.doe@example.com>",
        "date": "2025-10-27T10:30:00Z",
        "message": "feat: Add new feature

This commit introduces a brand new feature to the application.",
        "tags": ["v1.2.0", "release-candidate"],
        "signatureInfo": "Good signature from John Doe"
    })
}

#[tokio::test]
async fn test_revision_metadata_basic() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "my-app";
    let revision = "abcdef123456";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_revision_metadata_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.author, Some("John Doe <john.doe@example.com>".to_string()));
    assert_eq!(summary.date, Some("2025-10-27T10:30:00Z".to_string()));
    assert_eq!(summary.message_short, Some("feat: Add new feature".to_string()));
    assert!(summary.message_full.unwrap().contains("This commit introduces a brand new feature"));
    assert_eq!(summary.tag_count, 2);
    assert_eq!(summary.tags, Some(vec!["v1.2.0".to_string(), "release-candidate".to_string()]));
    assert!(summary.is_signed);
    assert_eq!(summary.signature_summary, Some("Valid signature".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_revision_metadata_no_tags_no_signature() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "another-app";
    let revision = "fedcba654321";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "author": "Jane Smith <jane.smith@example.com>",
            "date": "2025-10-26T09:00:00Z",
            "message": "fix: Bug fix for login issue"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.author, Some("Jane Smith <jane.smith@example.com>".to_string()));
    assert_eq!(summary.date, Some("2025-10-26T09:00:00Z".to_string()));
    assert_eq!(summary.message_short, Some("fix: Bug fix for login issue".to_string()));
    assert_eq!(summary.tag_count, 0);
    assert_eq!(summary.tags, None);
    assert!(!summary.is_signed);
    assert_eq!(summary.signature_summary, None);

    Ok(())
}

#[tokio::test]
async fn test_revision_metadata_with_parameters() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "param-app";
    let revision = "1234567890ab";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .and(query_param("appNamespace", "argocd"))
        .and(query_param("project", "dev"))
        .and(query_param("sourceIndex", "0"))
        .and(query_param("versionId", "1"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "author": "Test User",
            "date": "2025-10-28T11:00:00Z",
            "message": "chore: Update dependencies"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
        Some("argocd".to_string()),
        Some("dev".to_string()),
        Some(0),
        Some(1),
    ).await?;

    assert_eq!(summary.author, Some("Test User".to_string()));
    assert_eq!(summary.message_short, Some("chore: Update dependencies".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_revision_metadata_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "nonexistent-app";
    let revision = "nonexistent-rev";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found",
            "message": "Application or revision not found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
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
async fn test_revision_metadata_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "auth-app";
    let revision = "auth-rev";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid authentication token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
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
async fn test_revision_metadata_server_error() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "error-app";
    let revision = "error-rev";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "Internal Server Error",
            "message": "Failed to retrieve revision metadata"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
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
async fn test_revision_metadata_empty_response() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "empty-app";
    let revision = "empty-rev";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({}))) // Empty JSON object
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.revision_metadata(
        app_name.to_string(),
        revision.to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.author, None);
    assert_eq!(summary.date, None);
    assert_eq!(summary.message_short, None);
    assert_eq!(summary.message_full, None);
    assert_eq!(summary.tag_count, 0);
    assert_eq!(summary.tags, None);
    assert!(!summary.is_signed);
    assert_eq!(summary.signature_summary, None);

    Ok(())
}

#[tokio::test]
async fn test_revision_metadata_full() -> Result<()> {
    let mock_server = MockServer::start().await;
    let app_name = "full-app";
    let revision = "full-rev";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/applications/{}/revisions/{}/metadata", app_name, revision)))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_revision_metadata_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let metadata = client.revision_metadata_full(
        app_name.to_string(),
        revision.to_string(),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(metadata.author, Some("John Doe <john.doe@example.com>".to_string()));
        assert_eq!(metadata.message, Some("feat: Add new feature\n\nThis commit introduces a brand new feature to the application.".to_string()));
        assert_eq!(metadata.tags, Some(vec!["v1.2.0".to_string(), "release-candidate".to_string()]));
        assert_eq!(metadata.signature_info, Some("Good signature from John Doe".to_string()));
    
        Ok(())
    }
    
    /// Mock response for application sync windows
    fn create_mock_sync_windows_response() -> serde_json::Value {
        json!({
            "windows": [
                {
                    "kind": "allow",
                    "schedule": "0 0 * * *", // Every day at midnight
                    "duration": "1h",
                    "applications": ["guestbook", "helm-app"],
                    "namespaces": ["default", "staging"],
                    "clusters": ["https://kubernetes.default.svc"],
                    "manualSyncEnabled": true,
                    "startTime": "2025-01-01T00:00:00Z",
                    "endTime": "2025-01-01T01:00:00Z"
                },
                {
                    "kind": "deny",
                    "schedule": "0 2 * * *", // Every day at 2 AM
                    "duration": "30m",
                    "applications": ["backend-api"],
                    "namespaces": ["backend-prod"],
                    "manualSyncEnabled": false,
                    "startTime": "2025-01-01T02:00:00Z",
                    "endTime": "2025-01-01T02:30:00Z"
                }
            ]
        })
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_basic() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "my-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_windows_response()))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let summary = client.get_application_sync_windows(
            app_name.to_string(),
            None,
            None,
        ).await?;
    
        assert_eq!(summary.total_windows, 2);
        assert_eq!(summary.windows[0].kind, Some("allow".to_string()));
        assert_eq!(summary.windows[0].schedule, Some("0 0 * * *".to_string()));
        assert_eq!(summary.windows[0].applications, Some(vec!["guestbook".to_string(), "helm-app".to_string()]));
        assert_eq!(summary.windows[1].kind, Some("deny".to_string()));
        assert_eq!(summary.windows[1].manual_sync_enabled, Some(false));
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_no_windows() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "no-windows-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"windows": []})))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let summary = client.get_application_sync_windows(
            app_name.to_string(),
            None,
            None,
        ).await?;
    
        assert_eq!(summary.total_windows, 0);
        assert!(summary.windows.is_empty());
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_with_parameters() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "param-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .and(query_param("appNamespace", "argocd"))
            .and(query_param("project", "dev"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "windows": [
                    {
                        "kind": "allow",
                        "schedule": "0 0 * * *",
                        "applications": ["param-app"]
                    }
                ]
            })))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let summary = client.get_application_sync_windows(
            app_name.to_string(),
            Some("argocd".to_string()),
            Some("dev".to_string()),
        ).await?;
    
        assert_eq!(summary.total_windows, 1);
        assert_eq!(summary.windows[0].applications, Some(vec!["param-app".to_string()]));
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_authentication_error() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "auth-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "Unauthorized",
                "message": "Invalid authentication token"
            })))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
        let result = client.get_application_sync_windows(
            app_name.to_string(),
            None,
            None,
        ).await;
    
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("401") || error_msg.contains("Unauthorized"));
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_not_found() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "nonexistent-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "Not Found",
                "message": "Application 'nonexistent-app' not found"
            })))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let result = client.get_application_sync_windows(
            app_name.to_string(),
            None,
            None,
        ).await;
    
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("404") || error_msg.contains("Not Found"));
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_server_error() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "error-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "error": "Internal Server Error",
                "message": "Failed to retrieve sync windows"
            })))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let result = client.get_application_sync_windows(
            app_name.to_string(),
            None,
            None,
        ).await;
    
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500") || error_msg.contains("Server Error"));
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_get_application_sync_windows_full() -> Result<()> {
        let mock_server = MockServer::start().await;
        let app_name = "full-app";
    
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/applications/{}/sync-windows", app_name)))
            .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_sync_windows_response()))
            .mount(&mock_server)
            .await;
    
        let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
        let response = client.get_application_sync_windows_full(
            app_name.to_string(),
            None,
            None,
        ).await?;
    
        assert_eq!(response.windows.len(), 2);
        assert_eq!(response.windows[0].kind, Some("allow".to_string()));
        assert_eq!(response.windows[1].schedule, Some("0 2 * * *".to_string()));
    
        Ok(())
    }
    