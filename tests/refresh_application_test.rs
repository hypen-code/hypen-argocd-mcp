use argocd_mcp_server::argocd_client::ArgocdClient;
use serde_json::json;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_refresh_application_sync_status_changes() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock response BEFORE refresh (without refresh parameter) - OutOfSync
    let app_before = json!({
        "metadata": {
            "name": "guestbook",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/argoproj/argocd-example-apps",
                "path": "guestbook",
                "targetRevision": "HEAD"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "OutOfSync",
                "revision": "abc123"
            }
        }
    });

    // Mock response AFTER refresh (with refresh=hard parameter) - Synced
    let app_after = json!({
        "metadata": {
            "name": "guestbook",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/argoproj/argocd-example-apps",
                "path": "guestbook",
                "targetRevision": "HEAD"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "abc123"
            }
        }
    });

    // Mock without refresh parameter
    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/guestbook"))
        .and(query_param("refresh", ""))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_before.clone()))
        .mount(&mock_server)
        .await;

    // Mock with refresh=hard parameter
    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/guestbook"))
        .and(query_param("refresh", "hard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_after))
        .mount(&mock_server)
        .await;

    // Also mock without any query params (first call)
    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/guestbook$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_before))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .refresh_application("guestbook".to_string(), None, None, None)
        .await;

    assert!(result.is_ok(), "Failed to refresh: {:?}", result.err());
    let summary = result.unwrap();

    assert_eq!(summary.application_name, "guestbook");
    assert_eq!(summary.refresh_type, "hard");
    assert_eq!(summary.sync_status_before, "OutOfSync");
    assert_eq!(summary.sync_status_after, "Synced");
    assert_eq!(summary.health_status_before, "Healthy");
    assert_eq!(summary.health_status_after, "Healthy");
    assert!(summary.sync_status_changed);
    assert!(!summary.health_status_changed);
    assert!(!summary.revision_changed);
}

#[tokio::test]
async fn test_refresh_application_no_changes() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Same response for both before and after
    let app_response = json!({
        "metadata": {
            "name": "stable-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/repo",
                "path": "app",
                "targetRevision": "main"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "def456"
            }
        }
    });

    // Mock both calls to return same response
    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/stable-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .expect(2) // Expect 2 calls - before and after
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .refresh_application("stable-app".to_string(), Some("hard".to_string()), None, None)
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    assert_eq!(summary.application_name, "stable-app");
    assert_eq!(summary.sync_status_before, "Synced");
    assert_eq!(summary.sync_status_after, "Synced");
    assert_eq!(summary.health_status_before, "Healthy");
    assert_eq!(summary.health_status_after, "Healthy");
    assert!(!summary.sync_status_changed);
    assert!(!summary.health_status_changed);
    assert!(!summary.revision_changed);
}

#[tokio::test]
async fn test_refresh_application_health_changes() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Before refresh - Progressing
    let app_before = json!({
        "metadata": {
            "name": "deploying-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/app",
                "path": "manifests",
                "targetRevision": "v2.0"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "production"
            }
        },
        "status": {
            "health": {
                "status": "Progressing"
            },
            "sync": {
                "status": "Synced",
                "revision": "xyz789"
            }
        }
    });

    // After refresh - Healthy
    let app_after = json!({
        "metadata": {
            "name": "deploying-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/app",
                "path": "manifests",
                "targetRevision": "v2.0"
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
                "status": "Synced",
                "revision": "xyz789"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/deploying-app$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_before))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/deploying-app"))
        .and(query_param("refresh", "hard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_after))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .refresh_application("deploying-app".to_string(), None, None, None)
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    assert_eq!(summary.health_status_before, "Progressing");
    assert_eq!(summary.health_status_after, "Healthy");
    assert!(summary.health_status_changed);
    assert!(!summary.sync_status_changed);
}

#[tokio::test]
async fn test_refresh_application_revision_changes() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Before refresh - old revision
    let app_before = json!({
        "metadata": {
            "name": "updating-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/app",
                "path": "app",
                "targetRevision": "main"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "old123"
            }
        }
    });

    // After refresh - new revision detected
    let app_after = json!({
        "metadata": {
            "name": "updating-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/app",
                "path": "app",
                "targetRevision": "main"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "OutOfSync",
                "revision": "new456"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/updating-app$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_before))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/updating-app"))
        .and(query_param("refresh", "hard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_after))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .refresh_application("updating-app".to_string(), None, None, None)
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();

    assert_eq!(summary.sync_revision_before, Some("old123".to_string()));
    assert_eq!(summary.sync_revision_after, Some("new456".to_string()));
    assert!(summary.revision_changed);
    assert!(summary.sync_status_changed); // Also changed from Synced to OutOfSync
}

#[tokio::test]
async fn test_refresh_application_normal_type() {
    // Start mock server
    let mock_server = MockServer::start().await;

    let app_response = json!({
        "metadata": {
            "name": "test-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/test/repo",
                "path": "app",
                "targetRevision": "HEAD"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "health": {
                "status": "Healthy"
            },
            "sync": {
                "status": "Synced",
                "revision": "abc"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/test-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .refresh_application(
            "test-app".to_string(),
            Some("normal".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.refresh_type, "normal");
}
