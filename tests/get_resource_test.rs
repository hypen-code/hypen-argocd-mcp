use argocd_mcp_server::argocd_client::ArgocdClient;
use serde_json::json;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_resource_pod() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetResource endpoint
    let manifest = r#"apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
  labels:
    app: test-app
    env: test
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  containers:
  - name: nginx
    image: nginx:latest
status:
  phase: Running
"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "test-pod"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    // Create client
    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    // Call get_resource
    let result = client
        .get_resource(
            "test-app".to_string(),
            Some("default".to_string()),
            "test-pod".to_string(),
            "v1".to_string(),
            None, // core resource, no group
            "Pod".to_string(),
            None,
            None,
        )
        .await;

    // Verify result
    assert!(result.is_ok(), "Failed to get resource: {:?}", result.err());
    let summary = result.unwrap();
    assert_eq!(summary.app_name, "test-app");
    assert_eq!(summary.kind, "Pod");
    assert_eq!(summary.resource_name, "test-pod");
    assert_eq!(summary.namespace, Some("default".to_string()));
    assert_eq!(summary.version, "v1");
    assert!(summary.manifest.contains("kind: Pod"));
    assert!(summary.manifest.contains("test-pod"));

    // Verify parsed manifest summary
    assert_eq!(summary.manifest_summary.kind, Some("Pod".to_string()));
    assert_eq!(
        summary.manifest_summary.name,
        Some("test-pod".to_string())
    );
    assert_eq!(
        summary.manifest_summary.namespace,
        Some("default".to_string())
    );
}

#[tokio::test]
async fn test_get_resource_deployment() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetResource endpoint for deployment
    let manifest = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  namespace: production
  labels:
    app: nginx
  annotations:
    description: "Test deployment"
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.21
status:
  replicas: 3
  readyReplicas: 3
"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "nginx-deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_resource(
            "production-app".to_string(),
            Some("production".to_string()),
            "nginx-deployment".to_string(),
            "v1".to_string(),
            Some("apps".to_string()),
            "Deployment".to_string(),
            None,
            None,
        )
        .await;

    assert!(result.is_ok(), "Failed to get resource: {:?}", result.err());
    let summary = result.unwrap();
    assert_eq!(summary.kind, "Deployment");
    assert_eq!(summary.resource_name, "nginx-deployment");
    assert_eq!(summary.group, Some("apps".to_string()));
    assert!(summary.manifest.contains("kind: Deployment"));
    assert_eq!(
        summary.manifest_summary.kind,
        Some("Deployment".to_string())
    );
    assert_eq!(
        summary.manifest_summary.status_summary,
        Some("3/3 replicas ready".to_string())
    );
}

#[tokio::test]
async fn test_get_resource_with_labels() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the GetResource endpoint for service with labels
    let manifest = r#"apiVersion: v1
kind: Service
metadata:
  name: test-service
  namespace: default
  labels:
    app: test-app
    env: production
    version: "1.0"
    team: backend
    component: api
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: 8080
  selector:
    app: test-app
"#;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "test-service"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    let result = client
        .get_resource(
            "test-app".to_string(),
            Some("default".to_string()),
            "test-service".to_string(),
            "v1".to_string(),
            None,
            "Service".to_string(),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.manifest_summary.labels.is_some());
    let labels = summary.manifest_summary.labels.unwrap();
    assert!(labels.contains_key("app"));
    assert!(labels.contains_key("env"));
    assert_eq!(labels.get("env"), Some(&"production".to_string()));
}
