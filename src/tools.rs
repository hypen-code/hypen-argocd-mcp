use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars,
    tool, tool_handler, tool_router,
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

/// MCP Server handler for ArgoCD operations
#[derive(Clone)]
pub struct ArgocdMcpHandler {
    client: Arc<RwLock<Option<ArgocdClient>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ArgocdMcpHandler {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    /// Initialize the client with credentials
    pub async fn initialize(&self, base_url: String, access_token: String) -> anyhow::Result<()> {
        let client = ArgocdClient::new(base_url, access_token)?;
        let mut guard = self.client.write().await;
        *guard = Some(client);
        Ok(())
    }

    /// List ArgoCD applications with optional filters
    #[tool(description = "List ArgoCD applications. Returns optimized summaries including name, project, sync status, health status, repository information, and destination. Use filters to narrow down results by name, projects, labels, repository, or namespace.")]
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
            .list_applications(args.name, args.projects, args.selector, args.repo, args.app_namespace)
            .await
            .map_err(|e| McpError::internal_error(
                format!("Failed to list applications: {}", e),
                None
            ))?;

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
            let json_data = serde_json::to_string_pretty(&summaries)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None
                ))?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// List only application names for efficient name lookup and typo correction
    #[tool(description = "List only ArgoCD application names. Returns a simple list of all application names, which is extremely efficient for name lookups and auto-correcting typos in application names. Use this when you need to find exact application names or verify if an application exists.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to list application names: {}", e),
                None
            ))?;

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
    #[tool(description = "Perform server-side diff calculation for an ArgoCD application using dry-run apply. This executes a Server-Side Apply operation in dryrun mode and compares the predicted state with the live state. Returns a list of resources with their diff status, showing which resources have differences between the live and target state.")]
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
            .server_side_diff(args.app_name.clone(), args.app_namespace, args.project, args.target_manifests)
            .await
            .map_err(|e| McpError::internal_error(
                format!("Failed to perform server-side diff: {}", e),
                None
            ))?;

        if summaries.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("No resources found for application '{}'", args.app_name),
            )]))
        } else {
            // Count modified resources
            let modified_count = summaries.iter().filter(|s| s.modified).count();
            let total_count = summaries.len();

            // Format as readable text
            let mut output = format!(
                "Server-Side Diff for application '{}'\n",
                args.app_name
            );
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
                    output.push_str(&format!("{}. {} ({})", idx + 1, res.resource_name, res.kind));
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
                    output.push_str(&format!("{}. {} ({})", idx + 1, res.resource_name, res.kind));
                    if let Some(ns) = &res.namespace {
                        output.push_str(&format!(" in namespace '{}'", ns));
                    }
                    output.push('\n');
                }
            }

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summaries)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None
                ))?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get a single application by name
    #[tool(description = "Get detailed information about a specific ArgoCD application by name. Returns comprehensive application details including source repository, destination cluster, sync status, health status, and sync policy configuration. Use this when you need detailed information about a specific application.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to get application: {}", e),
                None
            ))?;

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

        if detail.auto_sync_enabled.is_some() || detail.auto_sync_prune.is_some() || detail.auto_sync_self_heal.is_some() {
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
        let json_data = serde_json::to_string_pretty(&detail)
            .map_err(|e| McpError::internal_error(
                format!("Failed to serialize response: {}", e),
                None
            ))?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// Get resource tree for an ArgoCD application
    #[tool(description = "Get the resource tree for an ArgoCD application. Returns a hierarchical view of all resources managed by the application, including Deployments, Services, Pods, ConfigMaps, and more. Provides resource counts by kind, health status summary, and sample resources. Use filters to narrow results by resource type, namespace, or other attributes.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to get resource tree: {}", e),
                None
            ))?;

        // Format as readable text
        let mut output = format!(
            "Resource Tree for application '{}'\n",
            args.application_name
        );
        output.push_str(&format!("Total resources: {}\n", summary.total_nodes));
        output.push_str(&format!("Orphaned resources: {}\n\n", summary.orphaned_nodes_count));

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
        let json_data = serde_json::to_string_pretty(&summary)
            .map_err(|e| McpError::internal_error(
                format!("Failed to serialize response: {}", e),
                None
            ))?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }

    /// List resource events for an ArgoCD application
    #[tool(description = "List Kubernetes events for an ArgoCD application or specific resource within an application. Returns event details including type (Normal/Warning), reason, message, timestamps, and involved objects. Use filters to narrow results by resource name, namespace, or UID. Events provide insights into application lifecycle, deployments, and issues.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to list resource events: {}", e),
                None
            ))?;

        if summary.total_events == 0 {
            Ok(CallToolResult::success(vec![Content::text(
                format!("No events found for application '{}'", args.application_name),
            )]))
        } else {
            // Format as readable text
            let mut output = format!(
                "Events for application '{}'\n",
                args.application_name
            );
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
            let json_data = serde_json::to_string_pretty(&summary)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None
                ))?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get pod logs for an ArgoCD application resource
    #[tool(description = "Get container logs from pods in an ArgoCD application. Supports filtering for errors/warnings, tailing logs, and analyzing log levels. Essential for troubleshooting deployments, investigating crashes, and monitoring application behavior. Use 'errors_only' parameter to filter for issues automatically.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to get pod logs: {}", e),
                None
            ))?;

        if summary.total_lines == 0 {
            let mut no_logs_msg = format!("No logs found for application '{}'", args.application_name);
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
            let mut output = format!(
                "Pod Logs for application '{}'\n",
                args.application_name
            );

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
            if summary.error_count > 0 || summary.warning_count > 0 || summary.potential_issue_count > 0 {
                output.push_str("\n📊 Log Analysis:\n");
                if summary.error_count > 0 {
                    output.push_str(&format!("  ❌ Errors: {}\n", summary.error_count));
                }
                if summary.warning_count > 0 {
                    output.push_str(&format!("  ⚠️  Warnings: {}\n", summary.warning_count));
                }
                if summary.potential_issue_count > 0 {
                    output.push_str(&format!("  🔍 Potential Issues: {}\n", summary.potential_issue_count));
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
            output.push_str(&format!("\n📝 Log Entries (showing {}):\n", summary.total_lines));
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
                    if entry.level.as_str() != "UNKNOWN" { ": " } else { "" }
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
            let json_data = serde_json::to_string_pretty(&summary)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None
                ))?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get application manifests
    #[tool(description = "Get Kubernetes manifests for an ArgoCD application. Returns parsed YAML/JSON manifests with metadata including kind, API version, name, and namespace. Useful for reviewing what will be deployed, validating configurations, and understanding application structure.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to get manifests: {}", e),
                None
            ))?;

        if summary.total_manifests == 0 {
            Ok(CallToolResult::success(vec![Content::text(
                format!("No manifests found for application '{}'", args.application_name),
            )]))
        } else {
            // Format as readable text
            let mut output = format!(
                "Manifests for application '{}'\n",
                args.application_name
            );

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
                    output.push_str(&format!("\nCommands used to generate manifests ({}):\n", commands.len()));
                    for (idx, cmd) in commands.iter().enumerate() {
                        output.push_str(&format!("  {}. {}\n", idx + 1, cmd));
                    }
                }
            }

            // List manifests
            output.push_str(&format!("\n📄 Manifest Summaries ({})\n", summary.total_manifests));
            output.push_str(&"─".repeat(80));
            output.push('\n');

            for (idx, manifest) in summary.manifests.iter().enumerate() {
                output.push_str(&format!("\n{}. {} - {}/{}\n",
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
                    output.push_str(&format!("   ... ({} more lines)\n", lines.len() - preview_lines));
                }
            }

            output.push_str("\n");
            output.push_str(&"─".repeat(80));
            output.push('\n');

            // Add helpful note
            output.push_str("\n💡 Tip: Use this to review what will be deployed and validate configurations\n");

            // Also include JSON for structured consumption
            let json_data = serde_json::to_string_pretty(&summary)
                .map_err(|e| McpError::internal_error(
                    format!("Failed to serialize response: {}", e),
                    None
                ))?;

            Ok(CallToolResult::success(vec![
                Content::text(output),
                Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
            ]))
        }
    }

    /// Get revision metadata
    #[tool(description = "Get metadata (author, date, message, tags) for a specific revision of an ArgoCD application. Returns commit information including author, timestamp, commit message, associated Git tags, and signature verification status. Useful for tracking changes, auditing deployments, and understanding revision history.")]
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
            .map_err(|e| McpError::internal_error(
                format!("Failed to get revision metadata: {}", e),
                None
            ))?;

        // Format as readable text
        let mut output = format!(
            "Revision Metadata for application '{}' at revision '{}'\n\n",
            args.application_name,
            args.revision
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
        let json_data = serde_json::to_string_pretty(&summary)
            .map_err(|e| McpError::internal_error(
                format!("Failed to serialize response: {}", e),
                None
            ))?;

        Ok(CallToolResult::success(vec![
            Content::text(output),
            Content::text(format!("\n--- JSON Data ---\n{}", json_data)),
        ]))
    }
}

#[tool_handler]
impl ServerHandler for ArgocdMcpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "argocd-mcp-server".to_string(),
                version: "0.1.0".to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some("ArgoCD MCP Server - provides tools to interact with ArgoCD API. Currently supports: list_applications (list and filter ArgoCD applications with full details), list_application_names (get only application names for efficient name lookup and typo correction), get_application (get detailed information about a specific application by name), server_side_diff (perform server-side diff calculation using dry-run apply to compare live and target states), resource_tree (get hierarchical resource tree view with health status and resource details), list_resource_events (list Kubernetes events for applications or specific resources with filtering capabilities), pod_logs (get container logs with intelligent error/warning filtering and log level analysis), get_manifests (get Kubernetes manifests with parsing and analysis). Set ARGOCD_BASE_URL and ARGOCD_ACCESS_TOKEN environment variables before starting.".to_string()),
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
