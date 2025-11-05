use argocd_mcp_server::argocd_client::ArgocdClient;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_application_history_with_entries() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetApplication endpoint with history
    let app_response = json!({
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
            },
            "history": [
                {
                    "id": 1,
                    "revision": "abc123def456789012345678901234567890",
                    "deployedAt": "2025-01-01T10:00:00Z",
                    "deployStartedAt": "2025-01-01T09:59:00Z",
                    "source": {
                        "repoURL": "https://github.com/argoproj/argocd-example-apps",
                        "path": "guestbook",
                        "targetRevision": "main"
                    },
                    "initiatedBy": {
                        "username": "admin",
                        "automated": false
                    }
                },
                {
                    "id": 2,
                    "revision": "def456abc789012345678901234567890123",
                    "deployedAt": "2025-01-02T14:30:00Z",
                    "source": {
                        "repoURL": "https://github.com/argoproj/argocd-example-apps",
                        "path": "guestbook",
                        "targetRevision": "main"
                    },
                    "initiatedBy": {
                        "automated": true
                    }
                },
                {
                    "id": 3,
                    "revision": "ghi789jkl012345678901234567890123456",
                    "deployedAt": "2025-01-03T16:45:00Z",
                    "deployStartedAt": "2025-01-03T16:44:30Z",
                    "source": {
                        "repoURL": "https://github.com/argoproj/argocd-example-apps",
                        "path": "guestbook",
                        "targetRevision": "v2.0"
                    },
                    "initiatedBy": {
                        "username": "john.doe"
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/guestbook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_application_history("guestbook".to_string(), None, None)
        .await;

    assert!(result.is_ok(), "Failed to get history: {:?}", result.err());
    let history = result.unwrap();
    assert_eq!(history.application_name, "guestbook");
    assert_eq!(history.total_entries, 3);
    assert_eq!(history.entries.len(), 3);

    // Verify sorted by ID descending (newest first)
    assert_eq!(history.entries[0].id, 3);
    assert_eq!(history.entries[1].id, 2);
    assert_eq!(history.entries[2].id, 1);

    // Verify first entry details
    let first = &history.entries[0];
    assert_eq!(first.revision, "ghi789jk");
    assert_eq!(
        first.revision_full,
        "ghi789jkl012345678901234567890123456"
    );
    assert_eq!(first.deployed_at, "2025-01-03T16:45:00Z");
    assert!(first.deploy_duration.is_some());
    assert_eq!(
        first.source_repo,
        Some("https://github.com/argoproj/argocd-example-apps".to_string())
    );
    assert_eq!(first.source_path, Some("guestbook".to_string()));
    assert_eq!(first.source_target_revision, Some("v2.0".to_string()));
    assert_eq!(first.initiated_by, Some("john.doe".to_string()));
    assert!(!first.automated);

    // Verify automated deployment
    let second = &history.entries[1];
    assert_eq!(second.initiated_by, Some("Automated".to_string()));
    assert!(second.automated);

    // Verify manual deployment
    let third = &history.entries[2];
    assert_eq!(third.initiated_by, Some("admin".to_string()));
    assert!(!third.automated);
}

#[tokio::test]
async fn test_get_application_history_empty() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetApplication endpoint with no history
    let app_response = json!({
        "metadata": {
            "name": "new-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/repo",
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
                "status": "Progressing"
            },
            "sync": {
                "status": "OutOfSync"
            },
            "history": []
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/new-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_application_history("new-app".to_string(), None, None)
        .await;

    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.application_name, "new-app");
    assert_eq!(history.total_entries, 0);
    assert!(history.entries.is_empty());
}

#[tokio::test]
async fn test_get_application_history_multi_source() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetApplication endpoint with multi-source history
    let app_response = json!({
        "metadata": {
            "name": "multi-source-app",
            "namespace": "argocd"
        },
        "spec": {
            "sources": [
                {
                    "repoURL": "https://github.com/example/repo1",
                    "path": "app",
                    "targetRevision": "main"
                },
                {
                    "repoURL": "https://github.com/example/repo2",
                    "chart": "mychart",
                    "targetRevision": "1.0.0"
                }
            ],
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
                "status": "Synced"
            },
            "history": [
                {
                    "id": 1,
                    "deployedAt": "2025-01-01T10:00:00Z",
                    "sources": [
                        {
                            "repoURL": "https://github.com/example/repo1",
                            "path": "app",
                            "targetRevision": "main"
                        },
                        {
                            "repoURL": "https://github.com/example/repo2",
                            "chart": "mychart",
                            "targetRevision": "1.0.0"
                        }
                    ],
                    "revisions": ["abc123", "def456"]
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/multi-source-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_application_history("multi-source-app".to_string(), None, None)
        .await;

    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.total_entries, 1);

    let entry = &history.entries[0];
    // Should use first source from sources array
    assert_eq!(
        entry.source_repo,
        Some("https://github.com/example/repo1".to_string())
    );
    assert_eq!(entry.source_path, Some("app".to_string()));
}

#[tokio::test]
async fn test_get_application_history_no_status() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetApplication endpoint with no status
    let app_response = json!({
        "metadata": {
            "name": "no-status-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://github.com/example/repo",
                "path": "app",
                "targetRevision": "HEAD"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/no-status-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_application_history("no-status-app".to_string(), None, None)
        .await;

    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.application_name, "no-status-app");
    assert_eq!(history.total_entries, 0);
    assert!(history.entries.is_empty());
}

#[tokio::test]
async fn test_get_application_history_chart_source() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetApplication endpoint with Helm chart source
    let app_response = json!({
        "metadata": {
            "name": "helm-app",
            "namespace": "argocd"
        },
        "spec": {
            "source": {
                "repoURL": "https://charts.example.com",
                "chart": "my-chart",
                "targetRevision": "1.2.3"
            },
            "destination": {
                "server": "https://kubernetes.default.svc",
                "namespace": "default"
            }
        },
        "status": {
            "history": [
                {
                    "id": 1,
                    "revision": "1.2.3",
                    "deployedAt": "2025-01-01T10:00:00Z",
                    "source": {
                        "repoURL": "https://charts.example.com",
                        "chart": "my-chart",
                        "targetRevision": "1.2.3"
                    }
                }
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/helm-app"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_response))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_application_history("helm-app".to_string(), None, None)
        .await;

    assert!(result.is_ok());
    let history = result.unwrap();
    assert_eq!(history.total_entries, 1);

    let entry = &history.entries[0];
    assert_eq!(
        entry.source_repo,
        Some("https://charts.example.com".to_string())
    );
    // Should use chart instead of path
    assert_eq!(entry.source_path, Some("my-chart".to_string()));
    assert_eq!(entry.source_target_revision, Some("1.2.3".to_string()));
}
