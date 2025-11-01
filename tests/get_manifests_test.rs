use argocd_mcp_server::argocd_client::ArgocdClient;
use anyhow::Result;
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, query_param, header};

/// Mock response for get manifests
fn create_mock_manifests_response() -> serde_json::Value {
    json!({
        "manifests": [
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: guestbook-ui
  namespace: default
spec:
  replicas: 3"#,
            r#"apiVersion: v1
kind: Service
metadata:
  name: guestbook-ui
  namespace: default
spec:
  ports:
  - port: 80"#,
            r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: guestbook-config
  namespace: default
data:
  key: value"#
        ],
        "namespace": "default",
        "server": "https://kubernetes.default.svc",
        "revision": "abc123",
        "sourceType": "Directory",
        "commands": [
            "kustomize build ."
        ]
    })
}

#[tokio::test]
async fn test_get_manifests_basic() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/guestbook/manifests"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_manifests_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.get_manifests(
        "guestbook".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_manifests, 3);
    assert_eq!(*summary.manifests_by_kind.get("Deployment").unwrap(), 1);
    assert_eq!(*summary.manifests_by_kind.get("Service").unwrap(), 1);
    assert_eq!(*summary.manifests_by_kind.get("ConfigMap").unwrap(), 1);
    assert_eq!(summary.revision, Some("abc123".to_string()));
    assert_eq!(summary.namespace, Some("default".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_manifests_with_revision() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/my-app/manifests"))
        .and(query_param("revision", "v1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_mock_manifests_response()))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.get_manifests(
        "my-app".to_string(),
        Some("v1.0.0".to_string()),
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_manifests, 3);

    Ok(())
}

#[tokio::test]
async fn test_get_manifests_empty() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/empty-app/manifests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifests": [],
            "namespace": "default"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let summary = client.get_manifests(
        "empty-app".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await?;

    assert_eq!(summary.total_manifests, 0);

    Ok(())
}

#[tokio::test]
async fn test_get_manifests_authentication_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/app/manifests"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "Unauthorized",
            "message": "Invalid token"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "invalid-token".to_string())?;
    let result = client.get_manifests(
        "app".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_get_manifests_not_found() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/applications/nonexistent/manifests"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found"
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string())?;
    let result = client.get_manifests(
        "nonexistent".to_string(),
        None,
        None,
        None,
        None,
        None,
    ).await;

    assert!(result.is_err());

    Ok(())
}
