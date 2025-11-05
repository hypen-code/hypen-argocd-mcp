use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::argocd_client::ArgocdClient;

/// Arguments for listing ArgoCD applications
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListApplicationsArgs {
    /// Filter by application name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Filter by project names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    /// Label selector to filter applications (e.g., 'env=prod')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Filter by repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Filter by application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
}

/// Arguments for listing only application names
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListApplicationNamesArgs {
    /// Filter by project names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    /// Label selector to filter applications (e.g., 'env=prod')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Filter by repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Filter by application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
}

/// Arguments for server-side diff calculation
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ServerSideDiffArgs {
    /// Application name (required)
    pub app_name: String,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Target manifests for comparison (array of YAML/JSON strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_manifests: Option<Vec<String>>,
}

/// Arguments for resource tree query
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ResourceTreeArgs {
    /// Application name (required)
    pub application_name: String,
    /// Resource namespace filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Resource name filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Resource version filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Resource group filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for get application
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetApplicationArgs {
    /// Application name (required)
    pub name: String,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Refresh mode: "normal" or "hard" to force refresh from repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<String>,
    /// Resource version for optimistic concurrency
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
}

/// Arguments for listing resource events
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListResourceEventsArgs {
    /// Application name (required)
    pub application_name: String,
    /// Resource namespace filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_namespace: Option<String>,
    /// Resource name filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    /// Resource UID filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uid: Option<String>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for getting pod logs
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PodLogsArgs {
    /// Application name (required)
    pub application_name: String,
    /// Pod namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Pod name (if not provided, must specify kind and resource_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_name: Option<String>,
    /// Container name (defaults to first container if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// Show logs since N seconds ago
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_seconds: Option<i64>,
    /// Number of lines from the end of the logs to show (default: 100 for context efficiency)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail_lines: Option<i64>,
    /// Show previous container logs (if container restarted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<bool>,
    /// Filter logs by text (server-side filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Resource kind (e.g., "Deployment", "StatefulSet")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Resource group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource name (alternative to pod_name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Filter to show only errors and potential issues (client-side filtering, recommended for LLM context)
    #[serde(default)]
    pub errors_only: bool,
}

/// Arguments for getting application manifests
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetManifestsArgs {
    /// Application name (required)
    pub application_name: String,
    /// Revision to get manifests for (defaults to current target revision)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Source positions (for multi-source applications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_positions: Option<Vec<i64>>,
    /// Revisions for multi-source applications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revisions: Option<Vec<String>>,
}

/// Arguments for getting revision metadata
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RevisionMetadataArgs {
    /// Application name (required)
    pub application_name: String,
    /// Revision/commit hash (required)
    pub revision: String,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Source index (for multi-source applications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_index: Option<i32>,
    /// Version ID from historical data (for multi-source applications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<i32>,
}

/// Arguments for getting application sync windows
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetApplicationSyncWindowsArgs {
    /// Application name (required)
    pub application_name: String,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for rolling back an application
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RollbackApplicationArgs {
    /// Application name (required)
    pub application_name: String,
    /// History ID to rollback to (required). If not specified or set to 0, will rollback to the previous version.
    pub id: i64,
    /// Dry run mode - if true, will preview the rollback without actually performing it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Whether to prune resources that are no longer defined in the target revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Sync resource specification for partial sync
#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct SyncResourceArgs {
    /// Resource group (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind (required, e.g., "Deployment", "Service")
    pub kind: String,
    /// Resource name (required)
    pub name: String,
    /// Resource namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Retry configuration for sync operations
#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct RetryArgs {
    /// Maximum number of retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Backoff duration (e.g., "5s", "1m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_duration: Option<String>,
    /// Maximum backoff duration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_max_duration: Option<String>,
    /// Backoff factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_factor: Option<i64>,
}

/// Arguments for getting a resource
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetResourceArgs {
    /// Application name (required)
    pub application_name: String,
    /// Resource namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Resource name (required)
    pub resource_name: String,
    /// Resource version (required, e.g., "v1")
    pub version: String,
    /// Resource API group (optional, empty for core resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind (required, e.g., "Pod", "Service", "Deployment")
    pub kind: String,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for patching a resource
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PatchResourceArgs {
    /// Application name (required)
    pub application_name: String,
    /// Resource namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Resource name (required)
    pub resource_name: String,
    /// Resource version (required, e.g., "v1")
    pub version: String,
    /// Resource API group (optional, empty for core resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind (required, e.g., "Pod", "Service", "Deployment")
    pub kind: String,
    /// Patch content (required) - JSON patch document as string
    pub patch: String,
    /// Patch type (optional, e.g., "application/json-patch+json", "application/merge-patch+json", "application/strategic-merge-patch+json")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_type: Option<String>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for getting application deployment history
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetApplicationHistoryArgs {
    /// Application name (required)
    pub application_name: String,
    /// Application namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for refreshing an application
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RefreshApplicationArgs {
    /// Application name (required)
    pub application_name: String,
    /// Refresh type: "normal" or "hard" (optional, defaults to "hard")
    /// - "normal": Regular refresh from cache
    /// - "hard": Force refresh from Git repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_type: Option<String>,
    /// Application namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// Arguments for syncing an application
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SyncApplicationArgs {
    /// Application name (required)
    pub application_name: String,
    /// Specific revision to sync to (optional, defaults to target revision in app spec)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Dry run mode - if true, will preview the sync without actually performing it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Whether to prune resources that are no longer defined in Git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    /// Force sync - use force apply to override any conflicts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Specific resources to sync (if not specified, syncs all resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<SyncResourceArgs>>,
    /// Sync options (e.g., ["Validate=false", "CreateNamespace=true"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_options: Option<Vec<String>>,
    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryArgs>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// MCP Server handler for ArgoCD operations
#[derive(Clone)]
pub struct ArgocdMcpHandler {
    client: Arc<RwLock<Option<ArgocdClient>>>,
    tool_router: ToolRouter<Self>,
    read_only: bool,
}

#[tool_router]
impl ArgocdMcpHandler {
    /// Create a new handler with optional read-only mode
    pub fn new() -> Self {
        Self::with_read_only(false)
    }

    /// Create a new handler with explicit read-only mode
    pub fn with_read_only(read_only: bool) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
            read_only,
        }
    }

    /// Create a new handler from environment variables
    /// Reads ARGOCD_READ_ONLY environment variable (true/false, default: false)
    pub fn from_env() -> Self {
        let read_only = std::env::var("ARGOCD_READ_ONLY")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);
        Self::with_read_only(read_only)
    }

    /// Initialize the client with credentials
    pub async fn initialize(&self, base_url: String, access_token: String) -> anyhow::Result<()> {
        let client = ArgocdClient::new(base_url, access_token)?;
        let mut guard = self.client.write().await;
        *guard = Some(client);
        Ok(())
    }

    /// Check if the handler is in read-only mode
    #[allow(dead_code)]
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// List ArgoCD applications with optional filters
    #[tool(
        description = "List ArgoCD applications. Returns optimized summaries including name, project, sync status, health status, repository information, and destination. Use filters to narrow down results by name, projects, labels, repository, or namespace."
    )]
    async fn list_applications(
        &self,
        Parameters(args): Parameters<ListApplicationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summaries = client
            .list_applications(
                args.name,
                args.projects,
                args.selector,
                args.repo,
                args.app_namespace,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to list applications: {}", e), None)
            })?;

        if summaries.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(
                "No applications found matching the criteria",
            )]))
        } else {
            // Format as readable text
            let mut output = format!("Found {} application(s):\n\n", summaries.len());

            for (idx, app) in summaries.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", idx + 1, app.name));
                if let Some(project) = &app.project {
                    output.push_str(&format!("   Project: {}\n", project));
                }
                if let Some(repo) = &app.repo_url {
                    output.push_str(&format!("   Repository: {}\n", repo));
                }
                if let Some(revision) = &app.target_revision {
                    output.push_str(&format!("   Target Revision: {}\n", revision));
                }
                if let Some(dest_server) = &app.destination_server {
                    output.push_str(&format!("   Destination Server: {}\n", dest_server));
                }
                if let Some(dest_ns) = &app.destination_namespace {
                    output.push_str(&format!("   Destination Namespace: {}\n", dest_ns));
                }
                if let Some(sync) = &app.sync_status {
                    output.push_str(&format!("   Sync Status: {}\n", sync));
                }
                if let Some(health) = &app.health_status {
                    output.push_str(&format!("   Health Status: {}\n", health));
                }
                if let Some(true) = app.auto_sync {
                    output.push_str("   Auto Sync: Enabled\n");
                }
                output.push('\n');
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summaries).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// List only application names for efficient name lookup and typo correction
    #[tool(
        description = "List only ArgoCD application names. Returns a simple list of all application names, which is extremely efficient for name lookups and auto-correcting typos in application names. Use this when you need to find exact application names or verify if an application exists."
    )]
    async fn list_application_names(
        &self,
        Parameters(args): Parameters<ListApplicationNamesArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let names = client
            .list_application_names(args.projects, args.selector, args.repo, args.app_namespace)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to list application names: {}", e), None)
            })?;

        if names.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(
                "No applications found matching the criteria",
            )]))
        } else {
            // Format as a simple, readable list
            let mut output = format!("Found {} application(s):\n\n", names.len());
            for (idx, name) in names.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", idx + 1, name));
            }

            // Also include as a simple array for easier parsing
            output.push_str("\n--- Application Names Array ---\n");
            output.push_str(&format!("{:?}", names));

            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
    }

    /// Perform server-side diff calculation using dry-run apply
    #[tool(
        description = "Perform server-side diff calculation for an ArgoCD application using dry-run apply. This executes a Server-Side Apply operation in dryrun mode and compares the predicted state with the live state. Returns a list of resources with their diff status, showing which resources have differences between the live and target state. NOTE: This feature requires ArgoCD v2.5+ with Server-Side Apply support. If unavailable, a 404 error will be returned."
    )]
    async fn server_side_diff(
        &self,
        Parameters(args): Parameters<ServerSideDiffArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summaries = client
            .server_side_diff(
                args.app_name.clone(),
                args.app_namespace,
                args.project,
                args.target_manifests,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to perform server-side diff: {}", e), None)
            })?;

        if summaries.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No resources found for application '{}'",
                args.app_name
            ))]))
        } else {
            // Count modified resources
            let modified_count = summaries.iter().filter(|s| s.modified).count();
            let total_count = summaries.len();

            // Format as readable text
            let mut output = format!("Server-Side Diff for application '{}'\n", args.app_name);
            output.push_str(&format!(
                "Total resources: {}, Modified: {}, In sync: {}\n\n",
                total_count,
                modified_count,
                total_count - modified_count
            ));

            // Group by modified status
            output.push_str("Modified Resources:\n");
            let modified: Vec<_> = summaries.iter().filter(|s| s.modified).collect();
            if modified.is_empty() {
                output.push_str("  (none)\n\n");
            } else {
                for (idx, res) in modified.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. {} ({})",
                        idx + 1,
                        res.resource_name,
                        res.kind
                    ));
                    if let Some(ns) = &res.namespace {
                        output.push_str(&format!(" in namespace '{}'", ns));
                    }
                    output.push('\n');
                    if let Some(summary) = &res.diff_summary {
                        output.push_str(&format!("   Status: {}\n", summary));
                    }
                }
                output.push('\n');
            }

            output.push_str("In Sync Resources:\n");
            let in_sync: Vec<_> = summaries.iter().filter(|s| !s.modified).collect();
            if in_sync.is_empty() {
                output.push_str("  (none)\n");
            } else {
                for (idx, res) in in_sync.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. {} ({})",
                        idx + 1,
                        res.resource_name,
                        res.kind
                    ));
                    if let Some(ns) = &res.namespace {
                        output.push_str(&format!(" in namespace '{}'", ns));
                    }
                    output.push('\n');
                }
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summaries).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get a single application by name
    #[tool(
        description = "Get detailed information about a specific ArgoCD application by name. Returns comprehensive application details including source repository, destination cluster, sync status, health status, and sync policy configuration. Use this when you need detailed information about a specific application."
    )]
    async fn get_application(
        &self,
        Parameters(args): Parameters<GetApplicationArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let detail = client
            .get_application(
                args.name.clone(),
                args.app_namespace,
                args.project,
                args.refresh,
                args.resource_version,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get application: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!("Application: {}\n\n", detail.name);

        if let Some(ns) = &detail.namespace {
            output.push_str(&format!("Namespace: {}\n", ns));
        }
        if let Some(project) = &detail.project {
            output.push_str(&format!("Project: {}\n", project));
        }
        if let Some(created) = &detail.creation_timestamp {
            output.push_str(&format!("Created: {}\n", created));
        }

        output.push_str("\nSource:\n");
        if let Some(repo) = &detail.repo_url {
            output.push_str(&format!("  Repository: {}\n", repo));
        }
        if let Some(path) = &detail.path {
            output.push_str(&format!("  Path: {}\n", path));
        }
        if let Some(chart) = &detail.chart {
            output.push_str(&format!("  Chart: {}\n", chart));
        }
        if let Some(revision) = &detail.target_revision {
            output.push_str(&format!("  Target Revision: {}\n", revision));
        }

        output.push_str("\nDestination:\n");
        if let Some(server) = &detail.destination_server {
            output.push_str(&format!("  Server: {}\n", server));
        }
        if let Some(ns) = &detail.destination_namespace {
            output.push_str(&format!("  Namespace: {}\n", ns));
        }
        if let Some(name) = &detail.destination_name {
            output.push_str(&format!("  Name: {}\n", name));
        }

        output.push_str("\nStatus:\n");
        if let Some(sync_status) = &detail.sync_status {
            output.push_str(&format!("  Sync Status: {}\n", sync_status));
        }
        if let Some(revision) = &detail.sync_revision {
            output.push_str(&format!("  Sync Revision: {}\n", revision));
        }
        if let Some(health) = &detail.health_status {
            output.push_str(&format!("  Health Status: {}\n", health));
        }
        if let Some(msg) = &detail.health_message {
            output.push_str(&format!("  Health Message: {}\n", msg));
        }

        if detail.auto_sync_enabled.is_some()
            || detail.auto_sync_prune.is_some()
            || detail.auto_sync_self_heal.is_some()
        {
            output.push_str("\nSync Policy:\n");
            if let Some(true) = detail.auto_sync_enabled {
                output.push_str("  Auto Sync: Enabled\n");
            }
            if let Some(prune) = detail.auto_sync_prune {
                output.push_str(&format!("  Auto Prune: {}\n", prune));
            }
            if let Some(self_heal) = detail.auto_sync_self_heal {
                output.push_str(&format!("  Self Heal: {}\n", self_heal));
            }
        }

        if let Some(labels) = &detail.labels {
            if !labels.is_empty() {
                output.push_str("\nLabels:\n");
                for (key, value) in labels {
                    output.push_str(&format!("  {}: {}\n", key, value));
                }
            }
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&detail).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Get resource tree for an ArgoCD application
    #[tool(
        description = "Get the resource tree for an ArgoCD application. Returns a hierarchical view of all resources managed by the application, including Deployments, Services, Pods, ConfigMaps, and more. Provides resource counts by kind, health status summary, and sample resources. Use filters to narrow results by resource type, namespace, or other attributes."
    )]
    async fn resource_tree(
        &self,
        Parameters(args): Parameters<ResourceTreeArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .resource_tree(
                args.application_name.clone(),
                args.namespace,
                args.name,
                args.version,
                args.group,
                args.kind,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get resource tree: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "Resource Tree for application '{}'\n",
            args.application_name
        );
        output.push_str(&format!("Total resources: {}\n", summary.total_nodes));
        output.push_str(&format!(
            "Orphaned resources: {}\n\n",
            summary.orphaned_nodes_count
        ));

        // Resources by kind
        output.push_str("Resources by Kind:\n");
        let mut kinds: Vec<_> = summary.nodes_by_kind.iter().collect();
        kinds.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        for (kind, count) in kinds {
            output.push_str(&format!("  {}: {}\n", kind, count));
        }
        output.push('\n');

        // Health summary
        output.push_str("Health Summary:\n");
        let mut health: Vec<_> = summary.health_summary.iter().collect();
        health.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
        for (status, count) in health {
            output.push_str(&format!("  {}: {}\n", status, count));
        }
        output.push('\n');

        // Sample nodes
        if !summary.sample_nodes.is_empty() {
            output.push_str("Sample Resources (showing up to 10):\n");
            for (idx, node) in summary.sample_nodes.iter().enumerate() {
                output.push_str(&format!("{}. {} ({})", idx + 1, node.name, node.kind));
                if let Some(ns) = &node.namespace {
                    output.push_str(&format!(" in namespace '{}'", ns));
                }
                if let Some(health) = &node.health_status {
                    output.push_str(&format!(" - Health: {}", health));
                }
                if let Some(parent_count) = node.parent_count {
                    if parent_count > 0 {
                        output.push_str(&format!(" - {} parent(s)", parent_count));
                    }
                }
                if let Some(images) = &node.images {
                    if !images.is_empty() {
                        output.push_str(&format!("\n   Images: {}", images.join(", ")));
                    }
                }
                output.push('\n');
            }
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// List resource events for an ArgoCD application
    #[tool(
        description = "List Kubernetes events for an ArgoCD application or specific resource within an application. Returns event details including type (Normal/Warning), reason, message, timestamps, and involved objects. Use filters to narrow results by resource name, namespace, or UID. Events provide insights into application lifecycle, deployments, and issues. NOTE: If no events are found or the response format is unexpected, an empty list will be returned with appropriate logging."
    )]
    async fn list_resource_events(
        &self,
        Parameters(args): Parameters<ListResourceEventsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .list_resource_events(
                args.application_name.clone(),
                args.resource_namespace,
                args.resource_name,
                args.resource_uid,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to list resource events: {}", e), None)
            })?;

        if summary.total_events == 0 {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No events found for application '{}'",
                args.application_name
            ))]))
        } else {
            // Format as readable text
            let mut output = format!("Events for application '{}'\n", args.application_name);
            output.push_str(&format!("Total events: {}\n\n", summary.total_events));

            // Event counts by type
            if !summary.events_by_type.is_empty() {
                output.push_str("Events by Type:\n");
                let mut types: Vec<_> = summary.events_by_type.iter().collect();
                types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                for (event_type, count) in types {
                    output.push_str(&format!("  {}: {}\n", event_type, count));
                }
                output.push('\n');
            }

            // Event counts by reason
            if !summary.events_by_reason.is_empty() {
                output.push_str("Events by Reason:\n");
                let mut reasons: Vec<_> = summary.events_by_reason.iter().collect();
                reasons.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                for (reason, count) in reasons.iter().take(10) {
                    output.push_str(&format!("  {}: {}\n", reason, count));
                }
                if reasons.len() > 10 {
                    output.push_str(&format!("  ... and {} more\n", reasons.len() - 10));
                }
                output.push('\n');
            }

            // List individual events (limited to prevent context overflow)
            let event_limit = 20;
            output.push_str(&format!("Recent Events (showing up to {}):\n", event_limit));
            for (idx, event) in summary.events.iter().take(event_limit).enumerate() {
                output.push_str(&format!("\n{}. ", idx + 1));

                if let Some(event_type) = &event.event_type {
                    output.push_str(&format!("[{}] ", event_type));
                }

                if let Some(reason) = &event.reason {
                    output.push_str(&format!("{}", reason));
                }

                if let Some(involved_kind) = &event.involved_object_kind {
                    output.push_str(&format!(" - {}", involved_kind));
                    if let Some(involved_name) = &event.involved_object_name {
                        output.push_str(&format!("/{}", involved_name));
                    }
                }

                output.push('\n');

                if let Some(message) = &event.message {
                    output.push_str(&format!("   Message: {}\n", message));
                }

                if let Some(count) = event.count {
                    if count > 1 {
                        output.push_str(&format!("   Count: {}\n", count));
                    }
                }

                if let Some(first_ts) = &event.first_timestamp {
                    output.push_str(&format!("   First: {}", first_ts));
                    if let Some(last_ts) = &event.last_timestamp {
                        if first_ts != last_ts {
                            output.push_str(&format!(" | Last: {}", last_ts));
                        }
                    }
                    output.push('\n');
                }

                if let Some(source) = &event.source_component {
                    output.push_str(&format!("   Source: {}\n", source));
                }
            }

            if summary.total_events > event_limit {
                output.push_str(&format!(
                    "\n... and {} more events (total: {})\n",
                    summary.total_events - event_limit,
                    summary.total_events
                ));
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get pod logs for an ArgoCD application resource
    #[tool(
        description = "Get container logs from pods in an ArgoCD application. Supports filtering for errors/warnings, tailing logs, and analyzing log levels. Essential for troubleshooting deployments, investigating crashes, and monitoring application behavior. Use 'errors_only' parameter to filter for issues automatically."
    )]
    async fn pod_logs(
        &self,
        Parameters(args): Parameters<PodLogsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Default tail_lines to 100 if not specified (context-efficient default)
        let tail_lines = args.tail_lines.or(Some(100));

        // Call ArgoCD API
        let summary = client
            .pod_logs(
                args.application_name.clone(),
                args.namespace,
                args.pod_name.clone(),
                args.container.clone(),
                args.since_seconds,
                tail_lines,
                args.previous,
                args.filter,
                args.kind,
                args.group,
                args.resource_name,
                args.app_namespace,
                args.project,
                args.errors_only,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get pod logs: {}", e), None)
            })?;

        if summary.total_lines == 0 {
            let mut no_logs_msg =
                format!("No logs found for application '{}'", args.application_name);
            if let Some(pod) = &args.pod_name {
                no_logs_msg.push_str(&format!(" pod '{}'", pod));
            }
            if let Some(container) = &args.container {
                no_logs_msg.push_str(&format!(" container '{}'", container));
            }
            if args.errors_only {
                no_logs_msg.push_str(" (no errors or warnings detected)");
            }

            Ok(CallToolResult::success(vec![Content::text(no_logs_msg)]))
        } else {
            // Format as readable text
            let mut output = format!("Pod Logs for application '{}'\n", args.application_name);

            if let Some(pod) = &summary.pod_name {
                output.push_str(&format!("Pod: {}\n", pod));
            }
            if let Some(container) = &summary.container {
                output.push_str(&format!("Container: {}\n", container));
            }
            if let Some(tail) = summary.tail_lines {
                output.push_str(&format!("Tail Lines: {}\n", tail));
            }

            output.push_str(&format!("\nTotal lines: {}\n", summary.total_lines));

            if summary.filtered {
                output.push_str("🔍 Filtered to show errors and potential issues only\n");
            }

            // Log statistics
            if summary.error_count > 0
                || summary.warning_count > 0
                || summary.potential_issue_count > 0
            {
                output.push_str("\n📊 Log Analysis:\n");
                if summary.error_count > 0 {
                    output.push_str(&format!("  ❌ Errors: {}\n", summary.error_count));
                }
                if summary.warning_count > 0 {
                    output.push_str(&format!("  ⚠️  Warnings: {}\n", summary.warning_count));
                }
                if summary.potential_issue_count > 0 {
                    output.push_str(&format!(
                        "  🔍 Potential Issues: {}\n",
                        summary.potential_issue_count
                    ));
                }
            }

            // Log levels breakdown
            if !summary.logs_by_level.is_empty() {
                output.push_str("\nLogs by Level:\n");
                let mut levels: Vec<_> = summary.logs_by_level.iter().collect();
                levels.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                for (level, count) in levels {
                    output.push_str(&format!("  {}: {}\n", level, count));
                }
            }

            // Show log entries
            output.push_str(&format!(
                "\n📝 Log Entries (showing {}):\n",
                summary.total_lines
            ));
            output.push_str(&"─".repeat(80));
            output.push('\n');

            for (idx, entry) in summary.log_entries.iter().enumerate() {
                // Add visual indicators for issues
                let indicator = match entry.level {
                    crate::models::LogLevel::Fatal => "💀",
                    crate::models::LogLevel::Error => "❌",
                    crate::models::LogLevel::Warning => "⚠️ ",
                    crate::models::LogLevel::Info => "ℹ️ ",
                    crate::models::LogLevel::Debug => "🐛",
                    crate::models::LogLevel::Unknown => "  ",
                };

                // Format timestamp if available
                let timestamp_str = if let Some(ts) = &entry.timestamp {
                    format!("[{}] ", ts)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}{} {}{}\n",
                    indicator,
                    timestamp_str,
                    entry.level.as_str(),
                    if entry.level.as_str() != "UNKNOWN" {
                        ": "
                    } else {
                        ""
                    }
                ));
                output.push_str(&format!("   {}\n", entry.content));

                // Add separator between entries for readability
                if idx < summary.total_lines - 1 {
                    output.push('\n');
                }
            }

            output.push_str(&"─".repeat(80));
            output.push('\n');

            // Add helpful notes
            if !summary.filtered && (summary.error_count > 0 || summary.potential_issue_count > 0) {
                output.push_str(&format!(
                    "\n💡 Tip: Use 'errors_only: true' to filter {} errors and {} potential issues\n",
                    summary.error_count, summary.potential_issue_count
                ));
            }

            if summary.total_lines >= 100 && tail_lines == Some(100) {
                output.push_str("\n💡 Tip: Increase 'tail_lines' to see more logs or use 'since_seconds' for time-based filtering\n");
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get application manifests
    #[tool(
        description = "Get Kubernetes manifests for an ArgoCD application. Returns parsed YAML/JSON manifests with metadata including kind, API version, name, and namespace. Useful for reviewing what will be deployed, validating configurations, and understanding application structure."
    )]
    async fn get_manifests(
        &self,
        Parameters(args): Parameters<GetManifestsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .get_manifests(
                args.application_name.clone(),
                args.revision,
                args.app_namespace,
                args.project,
                args.source_positions,
                args.revisions,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get manifests: {}", e), None)
            })?;

        if summary.total_manifests == 0 {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No manifests found for application '{}'",
                args.application_name
            ))]))
        } else {
            // Format as readable text
            let mut output = format!("Manifests for application '{}'\n", args.application_name);

            if let Some(rev) = &summary.revision {
                output.push_str(&format!("Revision: {}\n", rev));
            }
            if let Some(ns) = &summary.namespace {
                output.push_str(&format!("Namespace: {}\n", ns));
            }
            if let Some(server) = &summary.server {
                output.push_str(&format!("Server: {}\n", server));
            }
            if let Some(source_type) = &summary.source_type {
                output.push_str(&format!("Source Type: {}\n", source_type));
            }

            output.push_str(&format!("\nTotal manifests: {}\n", summary.total_manifests));

            // Manifests by kind
            if !summary.manifests_by_kind.is_empty() {
                output.push_str("\nManifests by Kind:\n");
                let mut kinds: Vec<_> = summary.manifests_by_kind.iter().collect();
                kinds.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
                for (kind, count) in kinds {
                    output.push_str(&format!("  {}: {}\n", kind, count));
                }
            }

            // Commands used
            if let Some(commands) = &summary.commands {
                if !commands.is_empty() {
                    output.push_str(&format!(
                        "\nCommands used to generate manifests ({}):\n",
                        commands.len()
                    ));
                    for (idx, cmd) in commands.iter().enumerate() {
                        output.push_str(&format!("  {}. {}\n", idx + 1, cmd));
                    }
                }
            }

            // List manifests
            output.push_str(&format!(
                "\n📄 Manifest Summaries ({})\n",
                summary.total_manifests
            ));
            output.push_str(&"─".repeat(80));
            output.push('\n');

            for (idx, manifest) in summary.manifests.iter().enumerate() {
                output.push_str(&format!(
                    "\n{}. {} - {}/{}\n",
                    idx + 1,
                    manifest.kind,
                    manifest.api_version,
                    manifest.name
                ));

                if let Some(ns) = &manifest.namespace {
                    output.push_str(&format!("   Namespace: {}\n", ns));
                }

                // Show first few lines of YAML for context (limited to prevent overflow)
                let lines: Vec<&str> = manifest.raw_yaml.lines().collect();
                let preview_lines = std::cmp::min(5, lines.len());
                output.push_str("   Preview:\n");
                for line in lines.iter().take(preview_lines) {
                    output.push_str(&format!("   {}\n", line));
                }
                if lines.len() > preview_lines {
                    output.push_str(&format!(
                        "   ... ({} more lines)\n",
                        lines.len() - preview_lines
                    ));
                }
            }

            output.push_str("\n");
            output.push_str(&"─".repeat(80));
            output.push('\n');

            // Add helpful note
            output.push_str(
                "\n💡 Tip: Use this to review what will be deployed and validate configurations\n",
            );

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get revision metadata
    #[tool(
        description = "Get metadata (author, date, message, tags) for a specific revision of an ArgoCD application. Returns commit information including author, timestamp, commit message, associated Git tags, and signature verification status. Useful for tracking changes, auditing deployments, and understanding revision history."
    )]
    async fn revision_metadata(
        &self,
        Parameters(args): Parameters<RevisionMetadataArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .revision_metadata(
                args.application_name.clone(),
                args.revision.clone(),
                args.app_namespace,
                args.project,
                args.source_index,
                args.version_id,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get revision metadata: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "Revision Metadata for application '{}' at revision '{}'\n\n",
            args.application_name, args.revision
        );

        if let Some(author) = &summary.author {
            output.push_str(&format!("Author: {}\n", author));
        }
        if let Some(date) = &summary.date {
            output.push_str(&format!("Date: {}\n", date));
        }

        // Show short message first
        if let Some(msg_short) = &summary.message_short {
            output.push_str(&format!("\nCommit Message:\n  {}\n", msg_short));
        }

        // Show full message if it's multi-line
        if let Some(msg_full) = &summary.message_full {
            let lines: Vec<&str> = msg_full.lines().collect();
            if lines.len() > 1 {
                output.push_str("\nFull Message:\n");
                for line in lines {
                    output.push_str(&format!("  {}\n", line));
                }
            }
        }

        // Tags information
        if summary.tag_count > 0 {
            output.push_str(&format!("\nTags ({}):\n", summary.tag_count));
            if let Some(tags) = &summary.tags {
                for tag in tags {
                    output.push_str(&format!("  - {}\n", tag));
                }
            }
        } else {
            output.push_str("\nTags: None\n");
        }

        // Signature information
        output.push_str("\nSignature Status: ");
        if summary.is_signed {
            if let Some(sig_summary) = &summary.signature_summary {
                output.push_str(&format!("{}\n", sig_summary));
            } else {
                output.push_str("Signed\n");
            }
        } else {
            output.push_str("Not signed\n");
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Get application sync windows
    #[tool(
        description = "Get synchronization windows for an ArgoCD application. Returns a list of configured sync windows, including their schedule, duration, and affected applications/namespaces/clusters. Useful for understanding when an application can be synced or is blocked from syncing. NOTE: This feature requires ArgoCD v2.6+. If unavailable, a 404 error will be returned."
    )]
    async fn get_application_sync_windows(
        &self,
        Parameters(args): Parameters<GetApplicationSyncWindowsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .get_application_sync_windows(
                args.application_name.clone(),
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to get application sync windows: {}", e),
                    None,
                )
            })?;

        if summary.total_windows == 0 {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No sync windows found for application '{}'",
                args.application_name
            ))]))
        } else {
            // Format as readable text
            let mut output = format!(
                "Sync Windows for application '{}' ({} total):\n\n",
                args.application_name, summary.total_windows
            );

            for (idx, window) in summary.windows.iter().enumerate() {
                output.push_str(&format!(
                    "{}. Kind: {}\n",
                    idx + 1,
                    window.kind.as_deref().unwrap_or("Unknown")
                ));
                if let Some(schedule) = &window.schedule {
                    output.push_str(&format!("   Schedule: {}\n", schedule));
                }
                if let Some(duration) = &window.duration {
                    output.push_str(&format!("   Duration: {}\n", duration));
                }
                if let Some(start_time) = &window.start_time {
                    output.push_str(&format!("   Start Time: {}\n", start_time));
                }
                if let Some(end_time) = &window.end_time {
                    output.push_str(&format!("   End Time: {}\n", end_time));
                }
                if let Some(manual_sync_enabled) = window.manual_sync_enabled {
                    output.push_str(&format!(
                        "   Manual Sync Enabled: {}\n",
                        manual_sync_enabled
                    ));
                }
                if let Some(apps) = &window.applications {
                    if !apps.is_empty() {
                        output.push_str(&format!("   Applications: {}\n", apps.join(", ")));
                    }
                }
                if let Some(namespaces) = &window.namespaces {
                    if !namespaces.is_empty() {
                        output.push_str(&format!("   Namespaces: {}\n", namespaces.join(", ")));
                    }
                }
                if let Some(clusters) = &window.clusters {
                    if !clusters.is_empty() {
                        output.push_str(&format!("   Clusters: {}\n", clusters.join(", ")));
                    }
                }
                output.push('\n');
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Rollback an application to a previous deployed version by History ID
    #[tool(
        description = "Rollback an ArgoCD application to a previous deployed version by History ID. This operation reverts the application to a specific point in its deployment history. Use dry_run mode to preview changes before applying. Use prune to remove resources that were removed in the target revision. Returns the application state after rollback including sync status, health status, and the revision that was rolled back to."
    )]
    async fn rollback_application(
        &self,
        Parameters(args): Parameters<RollbackApplicationArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Check if read-only mode is enabled
        if self.read_only {
            return Err(McpError::internal_error(
                "Cannot rollback application in read-only mode. This operation modifies application state.",
                None,
            ));
        }

        // Call ArgoCD API
        let summary = client
            .rollback_application(
                args.application_name.clone(),
                args.id,
                args.dry_run,
                args.prune,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to rollback application: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "Rollback {} for application '{}'\n\n",
            if summary.dry_run {
                "(Dry Run)"
            } else {
                "Completed"
            },
            summary.name
        );

        output.push_str(&format!(
            "Rolled back to History ID: {}\n",
            summary.rolled_back_to_id
        ));

        if let Some(target_rev) = &summary.target_revision {
            output.push_str(&format!("Target Revision: {}\n", target_rev));
        }

        if let Some(sync_rev) = &summary.sync_revision {
            output.push_str(&format!("Current Sync Revision: {}\n", sync_rev));
        }

        output.push_str("\nStatus:\n");
        if let Some(sync_status) = &summary.sync_status {
            output.push_str(&format!("  Sync Status: {}\n", sync_status));
        }
        if let Some(health_status) = &summary.health_status {
            output.push_str(&format!("  Health Status: {}\n", health_status));
        }

        output.push_str("\nOptions:\n");
        output.push_str(&format!("  Dry Run: {}\n", summary.dry_run));
        output.push_str(&format!("  Prune Enabled: {}\n", summary.prune_enabled));

        if summary.dry_run {
            output.push_str("\n⚠️  Note: This was a dry run. No actual changes were made.\n");
            output.push_str("    Run without dry_run=true to perform the actual rollback.\n");
        } else {
            output.push_str("\n✅ Rollback completed successfully.\n");
            output
                .push_str("    Monitor the application to ensure it reaches the desired state.\n");
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Sync an application to its target state
    #[tool(
        description = "Sync an ArgoCD application to its target state in Git. This operation deploys/updates the application resources to match what's defined in the Git repository. Supports dry-run mode to preview changes, selective resource sync, force sync to override conflicts, prune to remove orphaned resources, and custom sync options. Returns the application state after sync including sync status, health status, and applied configuration."
    )]
    async fn sync_application(
        &self,
        Parameters(args): Parameters<SyncApplicationArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Check if read-only mode is enabled
        if self.read_only {
            return Err(McpError::internal_error(
                "Cannot sync application in read-only mode. This operation modifies application state.",
                None,
            ));
        }

        // Convert args to model types
        let resources = args.resources.map(|res_args| {
            res_args
                .into_iter()
                .map(|r| crate::models::SyncResource {
                    group: r.group,
                    kind: r.kind,
                    name: r.name,
                    namespace: r.namespace,
                })
                .collect()
        });

        let retry = args.retry.map(|r| crate::models::RetryStrategy {
            limit: r.limit,
            backoff: if r.backoff_duration.is_some()
                || r.backoff_max_duration.is_some()
                || r.backoff_factor.is_some()
            {
                Some(crate::models::Backoff {
                    duration: r.backoff_duration,
                    max_duration: r.backoff_max_duration,
                    factor: r.backoff_factor,
                })
            } else {
                None
            },
        });

        // Call ArgoCD API
        let summary = client
            .sync_application(
                args.application_name.clone(),
                args.revision,
                args.dry_run,
                args.prune,
                args.force,
                resources,
                args.sync_options.clone(),
                retry,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to sync application: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "Sync {} for application '{}'\n\n",
            if summary.dry_run {
                "(Dry Run)"
            } else {
                "Completed"
            },
            summary.name
        );

        if let Some(rev) = &summary.target_revision {
            output.push_str(&format!("Target Revision: {}\n", rev));
        }

        if let Some(sync_rev) = &summary.sync_revision {
            output.push_str(&format!("Current Sync Revision: {}\n", sync_rev));
        }

        output.push_str("\nStatus:\n");
        if let Some(sync_status) = &summary.sync_status {
            output.push_str(&format!("  Sync Status: {}\n", sync_status));
        }
        if let Some(health_status) = &summary.health_status {
            output.push_str(&format!("  Health Status: {}\n", health_status));
        }

        output.push_str("\nConfiguration:\n");
        output.push_str(&format!("  Dry Run: {}\n", summary.dry_run));
        output.push_str(&format!("  Prune Enabled: {}\n", summary.prune_enabled));
        output.push_str(&format!("  Force Enabled: {}\n", summary.force_enabled));

        if let Some(count) = summary.resources_count {
            output.push_str(&format!("  Resources Synced: {} (partial sync)\n", count));
        } else {
            output.push_str("  Resources Synced: all\n");
        }

        if !summary.sync_options.is_empty() {
            output.push_str(&format!(
                "  Sync Options: {}\n",
                summary.sync_options.join(", ")
            ));
        }

        if summary.dry_run {
            output.push_str("\n⚠️  Note: This was a dry run. No actual changes were made.\n");
            output.push_str("    Run without dry_run=true to perform the actual sync.\n");
        } else {
            output.push_str("\n✅ Sync completed successfully.\n");
            output
                .push_str("    Monitor the application to ensure it reaches the desired state.\n");
            if summary.sync_status == Some("OutOfSync".to_string()) {
                output.push_str("    Note: Application may still be syncing. Check status again in a few moments.\n");
            }
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Get a specific resource from an ArgoCD application
    #[tool(
        description = "Get a specific Kubernetes resource from an ArgoCD application. Returns detailed resource manifest including metadata, spec, and status. Use this to inspect the current state of individual resources like Pods, Services, Deployments, ConfigMaps, etc. The manifest is parsed to extract key fields like API version, kind, name, namespace, labels, and status summary."
    )]
    async fn get_resource(
        &self,
        Parameters(args): Parameters<GetResourceArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let summary = client
            .get_resource(
                args.application_name.clone(),
                args.namespace.clone(),
                args.resource_name.clone(),
                args.version.clone(),
                args.group.clone(),
                args.kind.clone(),
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get resource: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "Resource: {} ({})\n",
            summary.resource_name, summary.kind
        );
        output.push_str(&format!("Application: {}\n", summary.app_name));
        output.push_str(&format!("Version: {}\n", summary.version));
        if let Some(group) = &summary.group {
            if !group.is_empty() {
                output.push_str(&format!("Group: {}\n", group));
            }
        }
        if let Some(ns) = &summary.namespace {
            output.push_str(&format!("Namespace: {}\n", ns));
        }

        output.push_str("\nManifest Summary:\n");
        if let Some(api_version) = &summary.manifest_summary.api_version {
            output.push_str(&format!("  API Version: {}\n", api_version));
        }
        if let Some(kind) = &summary.manifest_summary.kind {
            output.push_str(&format!("  Kind: {}\n", kind));
        }
        if let Some(name) = &summary.manifest_summary.name {
            output.push_str(&format!("  Name: {}\n", name));
        }
        if let Some(ns) = &summary.manifest_summary.namespace {
            output.push_str(&format!("  Namespace: {}\n", ns));
        }
        if let Some(labels) = &summary.manifest_summary.labels {
            if !labels.is_empty() {
                output.push_str(&format!("  Labels ({}): \n", labels.len()));
                for (key, value) in labels.iter().take(5) {
                    output.push_str(&format!("    {}: {}\n", key, value));
                }
                if labels.len() > 5 {
                    output.push_str(&format!("    ... and {} more\n", labels.len() - 5));
                }
            }
        }
        if let Some(count) = summary.manifest_summary.annotations_count {
            output.push_str(&format!("  Annotations Count: {}\n", count));
        }
        if let Some(created) = &summary.manifest_summary.creation_timestamp {
            output.push_str(&format!("  Created: {}\n", created));
        }
        if let Some(status) = &summary.manifest_summary.status_summary {
            output.push_str(&format!("  Status: {}\n", status));
        }

        output.push_str(&format!("\n📄 Full Manifest:\n{}\n", "─".repeat(80)));
        // Show first 50 lines of the manifest
        let lines: Vec<&str> = summary.manifest.lines().collect();
        let preview_lines = std::cmp::min(50, lines.len());
        for line in lines.iter().take(preview_lines) {
            output.push_str(&format!("{}\n", line));
        }
        if lines.len() > preview_lines {
            output.push_str(&format!(
                "\n... ({} more lines, {} total)\n",
                lines.len() - preview_lines,
                lines.len()
            ));
        }
        output.push_str(&"─".repeat(80));
        output.push('\n');

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Patch a specific resource in an ArgoCD application
    #[tool(
        description = "Patch a Kubernetes resource in an ArgoCD application. This operation modifies a specific resource using JSON patch, merge patch, or strategic merge patch formats. Returns the updated resource manifest. Common use cases include scaling deployments, updating environment variables, modifying labels/annotations, and changing resource configurations. NOTE: This is a write operation and is blocked in read-only mode."
    )]
    async fn patch_resource(
        &self,
        Parameters(args): Parameters<PatchResourceArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Check if read-only mode is enabled
        if self.read_only {
            return Err(McpError::internal_error(
                "Cannot patch resource in read-only mode. This operation modifies resource state.",
                None,
            ));
        }

        // Call ArgoCD API
        let summary = client
            .patch_resource(
                args.application_name.clone(),
                args.namespace.clone(),
                args.resource_name.clone(),
                args.version.clone(),
                args.group.clone(),
                args.kind.clone(),
                args.patch.clone(),
                args.patch_type,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to patch resource: {}", e), None)
            })?;

        // Format as readable text
        let mut output = format!(
            "✅ Patched Resource: {} ({})\n",
            summary.resource_name, summary.kind
        );
        output.push_str(&format!("Application: {}\n", summary.app_name));
        output.push_str(&format!("Version: {}\n", summary.version));
        if let Some(group) = &summary.group {
            if !group.is_empty() {
                output.push_str(&format!("Group: {}\n", group));
            }
        }
        if let Some(ns) = &summary.namespace {
            output.push_str(&format!("Namespace: {}\n", ns));
        }

        output.push_str("\nUpdated Manifest Summary:\n");
        if let Some(api_version) = &summary.manifest_summary.api_version {
            output.push_str(&format!("  API Version: {}\n", api_version));
        }
        if let Some(kind) = &summary.manifest_summary.kind {
            output.push_str(&format!("  Kind: {}\n", kind));
        }
        if let Some(name) = &summary.manifest_summary.name {
            output.push_str(&format!("  Name: {}\n", name));
        }
        if let Some(ns) = &summary.manifest_summary.namespace {
            output.push_str(&format!("  Namespace: {}\n", ns));
        }
        if let Some(labels) = &summary.manifest_summary.labels {
            if !labels.is_empty() {
                output.push_str(&format!("  Labels ({}): \n", labels.len()));
                for (key, value) in labels.iter().take(5) {
                    output.push_str(&format!("    {}: {}\n", key, value));
                }
                if labels.len() > 5 {
                    output.push_str(&format!("    ... and {} more\n", labels.len() - 5));
                }
            }
        }
        if let Some(count) = summary.manifest_summary.annotations_count {
            output.push_str(&format!("  Annotations Count: {}\n", count));
        }
        if let Some(status) = &summary.manifest_summary.status_summary {
            output.push_str(&format!("  Status: {}\n", status));
        }

        output.push_str(&format!("\n📄 Updated Manifest:\n{}\n", "─".repeat(80)));
        // Show first 50 lines of the manifest
        let lines: Vec<&str> = summary.manifest.lines().collect();
        let preview_lines = std::cmp::min(50, lines.len());
        for line in lines.iter().take(preview_lines) {
            output.push_str(&format!("{}\n", line));
        }
        if lines.len() > preview_lines {
            output.push_str(&format!(
                "\n... ({} more lines, {} total)\n",
                lines.len() - preview_lines,
                lines.len()
            ));
        }
        output.push_str(&"─".repeat(80));
        output.push('\n');

        output.push_str("\n💡 Tip: Monitor the resource to ensure it reaches the desired state.\n");

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Get application deployment history
    #[tool(
        description = "Get deployment history for an ArgoCD application. Returns a list of all deployments including revision, timestamp, source information, and who initiated each deployment. Essential for rollback operations (provides history IDs), auditing deployments, and understanding application changes over time. History is sorted newest first."
    )]
    async fn get_application_history(
        &self,
        Parameters(args): Parameters<GetApplicationHistoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let history = client
            .get_application_history(
                args.application_name.clone(),
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to get application history: {}", e),
                    None,
                )
            })?;

        // Format as readable text
        let mut output = format!(
            "📜 Deployment History for '{}'\n",
            history.application_name
        );
        output.push_str(&"═".repeat(80));
        output.push('\n');
        output.push_str(&format!("\nTotal deployments: {}\n", history.total_entries));

        if history.entries.is_empty() {
            output.push_str("\n⚠️  No deployment history available for this application.\n");
            output.push_str("   This could mean the application has never been synced.\n");
        } else {
            output.push_str("\n");

            // Show up to 20 most recent deployments
            let display_count = std::cmp::min(20, history.entries.len());
            for (idx, entry) in history.entries.iter().take(display_count).enumerate() {
                output.push_str(&"─".repeat(80));
                output.push('\n');

                let marker = if idx == 0 { "👉" } else { "  " };
                output.push_str(&format!(
                    "\n{} {}. History ID: {} {}\n",
                    marker,
                    idx + 1,
                    entry.id,
                    if idx == 0 { "(Current)" } else { "" }
                ));

                output.push_str(&format!("   Revision: {} ({})\n", entry.revision, entry.revision_full));
                output.push_str(&format!("   Deployed: {}\n", entry.deployed_at));

                if let Some(duration) = &entry.deploy_duration {
                    output.push_str(&format!("   Duration: {}\n", duration));
                }

                if let Some(initiator) = &entry.initiated_by {
                    let automated_marker = if entry.automated { " 🤖" } else { "" };
                    output.push_str(&format!("   Initiated by: {}{}\n", initiator, automated_marker));
                }

                if let Some(repo) = &entry.source_repo {
                    output.push_str(&format!("   Repository: {}\n", repo));
                }

                if let Some(path) = &entry.source_path {
                    output.push_str(&format!("   Path: {}\n", path));
                }

                if let Some(target) = &entry.source_target_revision {
                    output.push_str(&format!("   Target Revision: {}\n", target));
                }
            }

            output.push_str(&"─".repeat(80));
            output.push('\n');

            if history.entries.len() > display_count {
                output.push_str(&format!(
                    "\n... and {} more deployment(s) (showing {} most recent)\n",
                    history.entries.len() - display_count,
                    display_count
                ));
            }

            output.push_str("\n💡 Tips:\n");
            output.push_str("   - Use the History ID with 'rollback_application' to revert to a previous version\n");
            output.push_str("   - Current deployment is marked with 👉\n");
            output.push_str("   - 🤖 indicates automated deployments\n");
        }

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&history).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Refresh an application from Git repository
    #[tool(
        description = "Refresh an ArgoCD application from the Git repository. Forces ArgoCD to re-fetch the application manifests from Git and recompute the sync status. This is a read-only operation that does not modify cluster state - it only updates ArgoCD's cached view of the application. Use this to resolve stale sync status, update after Git changes, or troubleshoot 'stuck' applications. Returns before/after comparison showing what changed."
    )]
    async fn refresh_application(
        &self,
        Parameters(args): Parameters<RefreshApplicationArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Check if client is initialized
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            McpError::internal_error(
                "ArgoCD client not initialized. Please ensure ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables are set.",
                None,
            )
        })?;

        // Call ArgoCD API
        let refresh_summary = client
            .refresh_application(
                args.application_name.clone(),
                args.refresh_type,
                args.app_namespace,
                args.project,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to refresh application: {}", e),
                    None,
                )
            })?;

        // Format as readable text
        let mut output = format!(
            "🔄 Refreshed Application '{}'\n",
            refresh_summary.application_name
        );
        output.push_str(&"═".repeat(80));
        output.push('\n');

        output.push_str(&format!("\nRefresh Type: {}\n", refresh_summary.refresh_type));

        if let Some(repo) = &refresh_summary.repo_url {
            output.push_str(&format!("Repository: {}\n", repo));
        }
        if let Some(target) = &refresh_summary.target_revision {
            output.push_str(&format!("Target Revision: {}\n", target));
        }

        output.push_str("\n");
        output.push_str(&"─".repeat(80));
        output.push_str("\n\n");

        // Show before/after comparison
        output.push_str("📊 Status Comparison:\n\n");

        // Sync Status
        let sync_icon = if refresh_summary.sync_status_changed {
            "🔄"
        } else {
            "✓"
        };
        output.push_str(&format!("{}  Sync Status:\n", sync_icon));
        output.push_str(&format!("   Before: {}\n", refresh_summary.sync_status_before));
        output.push_str(&format!("   After:  {}\n", refresh_summary.sync_status_after));
        if refresh_summary.sync_status_changed {
            output.push_str("   ➜ Changed!\n");
        } else {
            output.push_str("   ➜ No change\n");
        }
        output.push('\n');

        // Health Status
        let health_icon = if refresh_summary.health_status_changed {
            "🔄"
        } else {
            "✓"
        };
        output.push_str(&format!("{}  Health Status:\n", health_icon));
        output.push_str(&format!("   Before: {}\n", refresh_summary.health_status_before));
        output.push_str(&format!("   After:  {}\n", refresh_summary.health_status_after));
        if refresh_summary.health_status_changed {
            output.push_str("   ➜ Changed!\n");
        } else {
            output.push_str("   ➜ No change\n");
        }
        output.push('\n');

        // Revision
        if refresh_summary.revision_changed {
            output.push_str("🔄  Sync Revision:\n");
            output.push_str(&format!(
                "   Before: {}\n",
                refresh_summary
                    .sync_revision_before
                    .as_ref()
                    .unwrap_or(&"None".to_string())
            ));
            output.push_str(&format!(
                "   After:  {}\n",
                refresh_summary
                    .sync_revision_after
                    .as_ref()
                    .unwrap_or(&"None".to_string())
            ));
            output.push_str("   ➜ Changed!\n\n");
        }

        output.push_str(&"─".repeat(80));
        output.push_str("\n\n");

        // Summary
        let any_changes = refresh_summary.sync_status_changed
            || refresh_summary.health_status_changed
            || refresh_summary.revision_changed;

        if any_changes {
            output.push_str("✅ Refresh completed - Application state was updated\n\n");

            let mut changes = Vec::new();
            if refresh_summary.sync_status_changed {
                changes.push("sync status");
            }
            if refresh_summary.health_status_changed {
                changes.push("health status");
            }
            if refresh_summary.revision_changed {
                changes.push("revision");
            }

            output.push_str(&format!("Changes detected in: {}\n", changes.join(", ")));
        } else {
            output.push_str("✅ Refresh completed - No changes detected\n\n");
            output.push_str("The application state in ArgoCD matches the Git repository.\n");
        }

        output.push_str("\n💡 Tips:\n");
        output.push_str("   - Refresh does not modify cluster resources, only ArgoCD's cache\n");
        output.push_str("   - Use 'hard' refresh to force re-fetch from Git repository\n");
        output.push_str("   - If sync status changed to 'OutOfSync', use 'sync_application' to deploy\n");

        // Also include JSON for structured consumption
        let json_data = serde_json::to_string_pretty(&refresh_summary).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize response: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }
}

#[tool_handler]
impl ServerHandler for ArgocdMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let mode_info = if self.read_only {
            " [READ-ONLY MODE: All tools are read-only GET requests only]"
        } else {
            ""
        };

        let instructions = format!(
            "ArgoCD MCP Server{} - provides tools to interact with ArgoCD API. Currently supports: list_applications (list and filter ArgoCD applications with full details), list_application_names (get only application names for efficient name lookup and typo correction), get_application (get detailed information about a specific application by name), server_side_diff (perform server-side diff calculation using dry-run apply to compare live and target states), resource_tree (get hierarchical resource tree view with health status and resource details), list_resource_events (list Kubernetes events for applications or specific resources with filtering capabilities), pod_logs (get container logs with intelligent error/warning filtering and log level analysis), get_manifests (get Kubernetes manifests with parsing and analysis), revision_metadata (get metadata for a specific revision including author, date, message, tags, and signature status), get_application_sync_windows (get synchronization windows for an application), get_application_history (get deployment history with history IDs for rollback operations), refresh_application (refresh application from Git repository without modifying cluster state), sync_application (sync an application to its target state in Git with dry-run, force, prune, and selective resource sync options), rollback_application (rollback an application to a previous deployed version by History ID with dry-run and prune options), get_resource (get a specific Kubernetes resource from an application with detailed manifest and parsed metadata), patch_resource (patch a Kubernetes resource in an application using JSON patch, merge patch, or strategic merge patch formats). Set ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables before starting. Optional: Set ARGOCD_READ_ONLY=true to enforce read-only mode (sync_application, rollback_application, and patch_resource are write operations and blocked in read-only mode).",
            mode_info
        );

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "argocd-mcp-server".to_string(),
                version: "0.1.0".to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(instructions),
        }
    }
}

impl Default for ArgocdMcpHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        let handler = ArgocdMcpHandler::new();
        assert!(handler.client.read().await.is_none());
    }

    #[tokio::test]
    async fn test_handler_initialization() {
        let handler = ArgocdMcpHandler::new();
        let result = handler
            .initialize(
                "https://argocd.example.com".to_string(),
                "test-token".to_string(),
            )
            .await;
        assert!(result.is_ok());
        assert!(handler.client.read().await.is_some());
    }
}
