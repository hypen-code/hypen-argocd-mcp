use argocd_mcp_server::argocd_client::ArgocdClient;
use serde_json::json;
use wiremock::matchers::{method, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_patch_resource_scale_deployment() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the PatchResource endpoint
    let manifest = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
  namespace: default
  labels:
    app: test-app
    patched: "true"
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  replicas: 5
  selector:
    matchLabels:
      app: test-app
  template:
    metadata:
      labels:
        app: test-app
    spec:
      containers:
      - name: nginx
        image: nginx:latest
status:
  replicas: 5
  readyReplicas: 5
"#;

    Mock::given(method("POST"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "test-deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    // Create client
    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    // Create a JSON merge patch to scale replicas
    let patch = json!({
        "spec": {
            "replicas": 5
        }
    })
    .to_string();

    // Call patch_resource
    let result = client
        .patch_resource(
            "test-app".to_string(),
            Some("default".to_string()),
            "test-deployment".to_string(),
            "v1".to_string(),
            Some("apps".to_string()),
            "Deployment".to_string(),
            patch,
            Some("application/merge-patch+json".to_string()),
            None,
            None,
        )
        .await;

    // Verify result
    assert!(result.is_ok(), "Failed to patch resource: {:?}", result.err());
    let summary = result.unwrap();
    assert_eq!(summary.app_name, "test-app");
    assert_eq!(summary.kind, "Deployment");
    assert_eq!(summary.resource_name, "test-deployment");
    assert!(summary.manifest.contains("replicas: 5"));
    assert!(summary.manifest.contains("kind: Deployment"));
}

#[tokio::test]
async fn test_patch_resource_add_label() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the PatchResource endpoint for adding label
    let manifest = r#"apiVersion: v1
kind: Service
metadata:
  name: test-service
  namespace: default
  labels:
    app: test-app
    environment: production
    patched: "true"
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  type: ClusterIP
  ports:
  - port: 80
  selector:
    app: test-app
"#;

    Mock::given(method("POST"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "test-service"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    // Create a strategic merge patch to add a label
    let patch = json!({
        "metadata": {
            "labels": {
                "environment": "production"
            }
        }
    })
    .to_string();

    let result = client
        .patch_resource(
            "test-app".to_string(),
            Some("default".to_string()),
            "test-service".to_string(),
            "v1".to_string(),
            None,
            "Service".to_string(),
            patch,
            Some("application/strategic-merge-patch+json".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.manifest.contains("environment: production"));
    assert!(summary.manifest_summary.labels.is_some());
}

#[tokio::test]
async fn test_patch_resource_json_patch() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the PatchResource endpoint for JSON patch
    let manifest = r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config
  namespace: default
  creationTimestamp: "2025-01-01T00:00:00Z"
data:
  config.json: |
    {"setting": "updated-value"}
  app.properties: |
    key=value
    updated=true
"#;

    Mock::given(method("POST"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "test-config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    // Create a JSON patch (RFC 6902)
    let patch = json!([
        {
            "op": "replace",
            "path": "/data/config.json",
            "value": "{\"setting\": \"updated-value\"}"
        }
    ])
    .to_string();

    let result = client
        .patch_resource(
            "test-app".to_string(),
            Some("default".to_string()),
            "test-config".to_string(),
            "v1".to_string(),
            None,
            "ConfigMap".to_string(),
            patch,
            Some("application/json-patch+json".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.manifest.contains("updated-value"));
    assert_eq!(summary.kind, "ConfigMap");
}

#[tokio::test]
async fn test_patch_resource_update_image() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Mock the PatchResource endpoint for updating image
    let manifest = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-deployment
  namespace: production
  creationTimestamp: "2025-01-01T00:00:00Z"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web
  template:
    metadata:
      labels:
        app: web
    spec:
      containers:
      - name: web
        image: nginx:1.22-alpine
        ports:
        - containerPort: 80
status:
  replicas: 3
  readyReplicas: 3
"#;

    Mock::given(method("POST"))
        .and(path_regex(r"/api/v1/applications/.*/resource"))
        .and(query_param("resourceName", "web-deployment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "manifest": manifest
        })))
        .mount(&mock_server)
        .await;

    let client = ArgocdClient::new(mock_server.uri(), "test-token".to_string()).unwrap();

    // Update container image
    let patch = json!({
        "spec": {
            "template": {
                "spec": {
                    "containers": [{
                        "name": "web",
                        "image": "nginx:1.22-alpine"
                    }]
                }
            }
        }
    })
    .to_string();

    let result = client
        .patch_resource(
            "prod-app".to_string(),
            Some("production".to_string()),
            "web-deployment".to_string(),
            "v1".to_string(),
            Some("apps".to_string()),
            "Deployment".to_string(),
            patch,
            Some("application/strategic-merge-patch+json".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.manifest.contains("nginx:1.22-alpine"));
    assert_eq!(summary.namespace, Some("production".to_string()));
}
