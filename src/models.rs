use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Optimized Application model containing only essential fields
/// to minimize context window usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ObjectMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ApplicationSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ApplicationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMeta {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ApplicationSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<ApplicationDestination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_policy: Option<SyncPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSource {
    #[serde(rename = "repoURL")]
    pub repo_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDestination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automated: Option<AutomatedSyncPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomatedSyncPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_heal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ApplicationSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<RevisionHistory>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ListMetadata>,
    #[serde(default)]
    pub items: Vec<Application>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
}

/// Optimized summary for context efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSummaryOutput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync: Option<bool>,
}

impl From<Application> for ApplicationSummaryOutput {
    fn from(app: Application) -> Self {
        let name = app
            .metadata
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default();

        let namespace = app.metadata.as_ref().and_then(|m| m.namespace.clone());

        let project = app.spec.as_ref().and_then(|s| s.project.clone());

        let repo_url = app
            .spec
            .as_ref()
            .and_then(|s| s.source.as_ref())
            .map(|src| src.repo_url.clone());

        let target_revision = app
            .spec
            .as_ref()
            .and_then(|s| s.source.as_ref())
            .and_then(|src| src.target_revision.clone());

        let destination_server = app
            .spec
            .as_ref()
            .and_then(|s| s.destination.as_ref())
            .and_then(|d| d.server.clone());

        let destination_namespace = app
            .spec
            .as_ref()
            .and_then(|s| s.destination.as_ref())
            .and_then(|d| d.namespace.clone());

        let sync_status = app
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone());

        let health_status = app
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|h| h.status.clone());

        let auto_sync = app
            .spec
            .as_ref()
            .and_then(|s| s.sync_policy.as_ref())
            .and_then(|sp| sp.automated.as_ref())
            .is_some();

        ApplicationSummaryOutput {
            name,
            namespace,
            project,
            repo_url,
            target_revision,
            destination_server,
            destination_namespace,
            sync_status,
            health_status,
            auto_sync: if auto_sync { Some(true) } else { None },
        }
    }
}

/// ResourceRef uniquely identifies a Kubernetes resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
}

/// InfoItem contains arbitrary name/value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// ResourceNetworkingInfo holds networking resource related information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceNetworkingInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_labels: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_refs: Option<Vec<ResourceRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress: Option<Vec<LoadBalancerIngress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls: Option<Vec<String>>,
}

/// LoadBalancerIngress represents the status of a load-balancer ingress point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadBalancerIngress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

/// ResourceNode contains information about a live resource and its relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceNode {
    // ResourceRef fields are inlined (json:",inline" in Go)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,

    // Additional ResourceNode fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_refs: Option<Vec<ResourceRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Vec<InfoItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking_info: Option<ResourceNetworkingInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// HostInfo holds information about physical resource nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_info: Option<HostSystemInfo>,
}

/// HostSystemInfo contains information about the host system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSystemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_runtime_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubelet_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kube_proxy_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
}

/// ApplicationTree holds the list of resource nodes that form the application tree
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationTree {
    #[serde(default)]
    pub nodes: Vec<ResourceNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orphaned_nodes: Option<Vec<ResourceNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<HostInfo>>,
}

/// Optimized summary for ResourceTree output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTreeSummary {
    pub total_nodes: usize,
    pub orphaned_nodes_count: usize,
    pub nodes_by_kind: std::collections::HashMap<String, usize>,
    pub health_summary: std::collections::HashMap<String, usize>,
    pub sample_nodes: Vec<ResourceNodeSummary>,
}

/// Optimized summary for a single resource node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNodeSummary {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

impl From<ApplicationTree> for ResourceTreeSummary {
    fn from(tree: ApplicationTree) -> Self {
        let total_nodes = tree.nodes.len();
        let orphaned_nodes_count = tree.orphaned_nodes.as_ref().map(|o| o.len()).unwrap_or(0);

        // Count nodes by kind
        let mut nodes_by_kind = std::collections::HashMap::new();
        for node in &tree.nodes {
            if let Some(kind) = &node.kind {
                *nodes_by_kind.entry(kind.clone()).or_insert(0) += 1;
            }
        }

        // Count nodes by health status
        let mut health_summary = std::collections::HashMap::new();
        for node in &tree.nodes {
            if let Some(health) = &node.health {
                *health_summary.entry(health.status.clone()).or_insert(0) += 1;
            } else {
                *health_summary.entry("Unknown".to_string()).or_insert(0) += 1;
            }
        }

        // Get sample nodes (limit to 10 to save context)
        let sample_nodes: Vec<ResourceNodeSummary> = tree
            .nodes
            .iter()
            .take(10)
            .map(|node| ResourceNodeSummary {
                name: node.name.clone().unwrap_or_default(),
                kind: node.kind.clone().unwrap_or_default(),
                namespace: node.namespace.clone(),
                health_status: node.health.as_ref().map(|h| h.status.clone()),
                parent_count: node.parent_refs.as_ref().map(|p| p.len()),
                images: node.images.clone(),
            })
            .collect();

        ResourceTreeSummary {
            total_nodes,
            orphaned_nodes_count,
            nodes_by_kind,
            health_summary,
            sample_nodes,
        }
    }
}

/// ResourceDiff represents the diff between a live and target resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDiff {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_live_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicted_live_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<bool>,
}

/// ApplicationServerSideDiffResponse contains the result of server-side diff calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationServerSideDiffResponse {
    #[serde(default)]
    pub items: Vec<ResourceDiff>,
    pub modified: bool,
}

/// Optimized summary for ServerSideDiff output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSideDiffSummary {
    pub resource_name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub modified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_summary: Option<String>,
}

impl From<ResourceDiff> for ServerSideDiffSummary {
    fn from(diff: ResourceDiff) -> Self {
        let resource_name = diff.name.clone().unwrap_or_default();
        let kind = diff.kind.clone().unwrap_or_default();
        let namespace = diff.namespace.clone();
        let modified = diff.modified.unwrap_or(false);

        // Create a simple summary of differences if modified
        let diff_summary = if modified {
            Some("Resource has differences between live and target state".to_string())
        } else {
            None
        };

        ServerSideDiffSummary {
            resource_name,
            kind,
            namespace,
            modified,
            diff_summary,
        }
    }
}

/// Optimized detail output for a single application (more detailed than summary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDetailOutput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync_prune: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync_self_heal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_timestamp: Option<String>,
}

impl From<Application> for ApplicationDetailOutput {
    fn from(app: Application) -> Self {
        let metadata = app.metadata.as_ref();
        let spec = app.spec.as_ref();
        let status = app.status.as_ref();

        let name = metadata.map(|m| m.name.clone()).unwrap_or_default();

        let namespace = metadata.and_then(|m| m.namespace.clone());

        let labels = metadata.and_then(|m| m.labels.clone());

        let creation_timestamp = metadata.and_then(|m| m.creation_timestamp.clone());

        let project = spec.and_then(|s| s.project.clone());

        let source = spec.and_then(|s| s.source.as_ref());
        let repo_url = source.map(|src| src.repo_url.clone());
        let path = source.and_then(|src| src.path.clone());
        let chart = source.and_then(|src| src.chart.clone());
        let target_revision = source.and_then(|src| src.target_revision.clone());

        let destination = spec.and_then(|s| s.destination.as_ref());
        let destination_server = destination.and_then(|d| d.server.clone());
        let destination_namespace = destination.and_then(|d| d.namespace.clone());
        let destination_name = destination.and_then(|d| d.name.clone());

        let sync = status.and_then(|s| s.sync.as_ref());
        let sync_status = sync.map(|s| s.status.clone());
        let sync_revision = sync.and_then(|s| s.revision.clone());

        let health = status.and_then(|s| s.health.as_ref());
        let health_status = health.map(|h| h.status.clone());
        let health_message = health.and_then(|h| h.message.clone());

        let sync_policy = spec.and_then(|s| s.sync_policy.as_ref());
        let automated = sync_policy.and_then(|sp| sp.automated.as_ref());
        let auto_sync_enabled = automated.is_some();
        let auto_sync_prune = automated.and_then(|a| a.prune);
        let auto_sync_self_heal = automated.and_then(|a| a.self_heal);

        ApplicationDetailOutput {
            name,
            namespace,
            project,
            repo_url,
            path,
            chart,
            target_revision,
            destination_server,
            destination_namespace,
            destination_name,
            sync_status,
            sync_revision,
            health_status,
            health_message,
            auto_sync_enabled: if auto_sync_enabled { Some(true) } else { None },
            auto_sync_prune,
            auto_sync_self_heal,
            labels,
            creation_timestamp,
        }
    }
}

/// Pod Logs structures for PodLogs2

/// LogEntry represents a single log line from a container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_stamp_str: Option<String>,
}

/// Log level for categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warning,
    Fatal,
    Info,
    Debug,
    Unknown,
}

impl LogLevel {
    /// Detect log level from log content
    pub fn detect(content: &str) -> Self {
        let content_upper = content.to_uppercase();

        if content_upper.contains("FATAL") || content_upper.contains("CRITICAL") {
            LogLevel::Fatal
        } else if content_upper.contains("ERROR") || content_upper.contains("ERR") {
            LogLevel::Error
        } else if content_upper.contains("WARN") || content_upper.contains("WARNING") {
            LogLevel::Warning
        } else if content_upper.contains("DEBUG") {
            LogLevel::Debug
        } else if content_upper.contains("INFO") {
            LogLevel::Info
        } else {
            LogLevel::Unknown
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Fatal => "FATAL",
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

/// Analyzed log entry with detected level and potential issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedLogEntry {
    pub content: String,
    pub level: LogLevel,
    pub pod_name: Option<String>,
    pub timestamp: Option<String>,
    pub is_error: bool,
    pub is_warning: bool,
    pub potential_issue: bool,
}

impl From<LogEntry> for AnalyzedLogEntry {
    fn from(entry: LogEntry) -> Self {
        let content = entry.content.unwrap_or_default();
        let level = LogLevel::detect(&content);
        let is_error = matches!(level, LogLevel::Error | LogLevel::Fatal);
        let is_warning = matches!(level, LogLevel::Warning);

        // Detect potential issues beyond explicit log levels
        let potential_issue = is_error
            || is_warning
            || content.to_lowercase().contains("exception")
            || content.to_lowercase().contains("failed")
            || content.to_lowercase().contains("timeout")
            || content.to_lowercase().contains("panic")
            || content.to_lowercase().contains("crash")
            || content.to_lowercase().contains("unable to")
            || content.to_lowercase().contains("cannot")
            || content.to_lowercase().contains("refused")
            || content.to_lowercase().contains("denied");

        AnalyzedLogEntry {
            content,
            level,
            pod_name: entry.pod_name,
            timestamp: entry.time_stamp_str.or(entry.time_stamp),
            is_error,
            is_warning,
            potential_issue,
        }
    }
}

/// Optimized pod logs summary for context efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodLogsSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub potential_issue_count: usize,
    pub logs_by_level: HashMap<String, usize>,
    pub pod_name: Option<String>,
    pub container: Option<String>,
    pub tail_lines: Option<i64>,
    pub filtered: bool,
    pub log_entries: Vec<AnalyzedLogEntry>,
}

impl PodLogsSummary {
    pub fn from_entries(
        entries: Vec<LogEntry>,
        pod_name: Option<String>,
        container: Option<String>,
        tail_lines: Option<i64>,
        filter_errors_only: bool,
    ) -> Self {
        let mut analyzed: Vec<AnalyzedLogEntry> =
            entries.into_iter().map(AnalyzedLogEntry::from).collect();

        // Apply error filtering if requested
        let filtered = filter_errors_only;
        if filter_errors_only {
            analyzed.retain(|entry| entry.potential_issue);
        }

        let total_lines = analyzed.len();
        let error_count = analyzed.iter().filter(|e| e.is_error).count();
        let warning_count = analyzed.iter().filter(|e| e.is_warning).count();
        let potential_issue_count = analyzed.iter().filter(|e| e.potential_issue).count();

        let mut logs_by_level = HashMap::new();
        for entry in &analyzed {
            *logs_by_level
                .entry(entry.level.as_str().to_string())
                .or_insert(0) += 1;
        }

        PodLogsSummary {
            total_lines,
            error_count,
            warning_count,
            potential_issue_count,
            logs_by_level,
            pod_name,
            container,
            tail_lines,
            filtered,
            log_entries: analyzed,
        }
    }
}

/// Manifest structures for GetManifests

/// ManifestResponse contains application manifests and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_result: Option<String>,
}

/// Parsed manifest object with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedManifest {
    pub kind: String,
    pub api_version: String,
    pub name: String,
    pub namespace: Option<String>,
    pub raw_yaml: String,
}

impl ParsedManifest {
    /// Parse a YAML manifest string into structured data
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        // Try to parse as JSON first (manifests can be JSON)
        let parsed: serde_json::Value =
            serde_yaml::from_str(yaml).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let kind = parsed["kind"].as_str().unwrap_or("Unknown").to_string();

        let api_version = parsed["apiVersion"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        let name = parsed["metadata"]["name"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        let namespace = parsed["metadata"]["namespace"]
            .as_str()
            .map(|s| s.to_string());

        Ok(ParsedManifest {
            kind,
            api_version,
            name,
            namespace,
            raw_yaml: yaml.to_string(),
        })
    }
}

/// Optimized manifest summary for context efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSummary {
    pub total_manifests: usize,
    pub manifests_by_kind: HashMap<String, usize>,
    pub revision: Option<String>,
    pub namespace: Option<String>,
    pub server: Option<String>,
    pub source_type: Option<String>,
    pub commands: Option<Vec<String>>,
    pub manifests: Vec<ParsedManifest>,
}

impl From<ManifestResponse> for ManifestSummary {
    fn from(response: ManifestResponse) -> Self {
        let manifests_raw = response.manifests.unwrap_or_default();
        let total_manifests = manifests_raw.len();

        // Parse all manifests
        let mut manifests = Vec::new();
        for yaml in manifests_raw {
            if let Ok(parsed) = ParsedManifest::from_yaml(&yaml) {
                manifests.push(parsed);
            }
        }

        // Count by kind
        let mut manifests_by_kind = HashMap::new();
        for manifest in &manifests {
            *manifests_by_kind.entry(manifest.kind.clone()).or_insert(0) += 1;
        }

        ManifestSummary {
            total_manifests,
            manifests_by_kind,
            revision: response.revision,
            namespace: response.namespace,
            server: response.server,
            source_type: response.source_type,
            commands: response.commands,
            manifests,
        }
    }
}

/// Kubernetes Event structures for ListResourceEvents

/// EventList is a list of events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ListMeta>,
    #[serde(default)]
    pub items: Vec<Event>,
}

/// ListMeta describes metadata for list resources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_item_count: Option<i64>,
}

/// Event is a report of an event somewhere in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EventMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub involved_object: Option<ObjectReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<EventSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<ObjectReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<EventSeries>,
}

/// EventMetadata contains metadata about the event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_timestamp: Option<String>,
}

/// ObjectReference contains enough information to let you inspect or modify the referred object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_path: Option<String>,
}

/// EventSource contains information for an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

/// EventSeries contains information on series of events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSeries {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_observed_time: Option<String>,
}

/// Optimized summary for EventList output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventListSummary {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub events_by_reason: HashMap<String, usize>,
    pub events: Vec<EventSummary>,
}

/// Optimized summary for a single event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub involved_object_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub involved_object_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub involved_object_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_timestamp: Option<String>,
}

impl From<EventList> for EventListSummary {
    fn from(event_list: EventList) -> Self {
        let total_events = event_list.items.len();

        // Count events by type
        let mut events_by_type = HashMap::new();
        for event in &event_list.items {
            if let Some(event_type) = &event.r#type {
                *events_by_type.entry(event_type.clone()).or_insert(0) += 1;
            }
        }

        // Count events by reason
        let mut events_by_reason = HashMap::new();
        for event in &event_list.items {
            if let Some(reason) = &event.reason {
                *events_by_reason.entry(reason.clone()).or_insert(0) += 1;
            }
        }

        // Convert events to summaries
        let events: Vec<EventSummary> = event_list
            .items
            .into_iter()
            .map(EventSummary::from)
            .collect();

        EventListSummary {
            total_events,
            events_by_type,
            events_by_reason,
            events,
        }
    }
}

impl From<Event> for EventSummary {
    fn from(event: Event) -> Self {
        let name = event.metadata.as_ref().and_then(|m| m.name.clone());
        let event_type = event.r#type.clone();
        let reason = event.reason.clone();
        let message = event.message.clone();

        let involved_object_kind = event
            .involved_object
            .as_ref()
            .and_then(|obj| obj.kind.clone());
        let involved_object_name = event
            .involved_object
            .as_ref()
            .and_then(|obj| obj.name.clone());
        let involved_object_namespace = event
            .involved_object
            .as_ref()
            .and_then(|obj| obj.namespace.clone());

        let source_component = event.source.as_ref().and_then(|src| src.component.clone());

        let count = event.count;
        let first_timestamp = event.first_timestamp.clone();
        let last_timestamp = event.last_timestamp.clone();

        EventSummary {
            name,
            event_type,
            reason,
            message,
            involved_object_kind,
            involved_object_name,
            involved_object_namespace,
            source_component,
            count,
            first_timestamp,
            last_timestamp,
        }
    }
}

/// Revision Metadata structures for RevisionMetadata endpoint

/// RevisionMetadata contains metadata about a specific revision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionMetadata {
    /// Author of the revision (e.g., git commit author)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Date/time of the revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Commit/revision message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Tags associated with the revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Signature information (e.g., GPG signature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_info: Option<String>,
}

/// Optimized summary for RevisionMetadata output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionMetadataSummary {
    /// Author of the revision
    pub author: Option<String>,
    /// Date/time of the revision
    pub date: Option<String>,
    /// Shortened commit message (first line)
    pub message_short: Option<String>,
    /// Full commit message
    pub message_full: Option<String>,
    /// Number of tags
    pub tag_count: usize,
    /// Tags associated with the revision
    pub tags: Option<Vec<String>>,
    /// Whether the revision is signed
    pub is_signed: bool,
    /// Signature information summary
    pub signature_summary: Option<String>,
}

impl From<RevisionMetadata> for RevisionMetadataSummary {
    fn from(metadata: RevisionMetadata) -> Self {
        let message_full = metadata.message.clone();
        let message_short = metadata
            .message
            .as_ref()
            .and_then(|msg| msg.lines().next().map(|s| s.to_string()));

        let tag_count = metadata.tags.as_ref().map(|t| t.len()).unwrap_or(0);
        let is_signed = metadata.signature_info.is_some();
        let signature_summary = metadata.signature_info.as_ref().and_then(|sig| {
            if sig.is_empty() {
                None
            } else if sig.contains("Good signature") || sig.to_uppercase().contains("VALID") {
                Some("Valid signature".to_string())
            } else if sig.contains("Bad signature") || sig.to_uppercase().contains("INVALID") {
                Some("Invalid signature".to_string())
            } else {
                Some("Signature present".to_string())
            }
        });

        RevisionMetadataSummary {
            author: metadata.author,
            date: metadata.date,
            message_short,
            message_full,
            tag_count,
            tags: metadata.tags,
            is_signed,
            signature_summary,
        }
    }
}

/// ApplicationSyncWindow represents a single sync window
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSyncWindow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>, // e.g., "allow", "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>, // Cron schedule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>, // e.g., "1h", "30m"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applications: Option<Vec<String>>, // List of application names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespaces: Option<Vec<String>>, // List of namespaces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters: Option<Vec<String>>, // List of cluster URLs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>, // RFC3339 format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>, // RFC3339 format
}

/// ApplicationSyncWindowsResponse is the full response from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSyncWindowsResponse {
    #[serde(default)]
    pub windows: Vec<ApplicationSyncWindow>,
}

/// Optimized summary for ApplicationSyncWindows output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSyncWindowsSummary {
    pub total_windows: usize,
    pub windows: Vec<ApplicationSyncWindow>, // Keep full windows for now, can optimize later if needed
}

impl From<ApplicationSyncWindowsResponse> for ApplicationSyncWindowsSummary {
    fn from(response: ApplicationSyncWindowsResponse) -> Self {
        let total_windows = response.windows.len();
        ApplicationSyncWindowsSummary {
            total_windows,
            windows: response.windows,
        }
    }
}

/// ApplicationRollbackRequest is the request for rolling back an application
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationRollbackRequest {
    /// Application name (required)
    pub name: String,
    /// History ID to rollback to (required)
    pub id: i64,
    /// Dry run mode - if true, will not actually perform the rollback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Whether to prune resources that are no longer defined in Git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// ApplicationRollbackResponse contains the result of a rollback operation
/// The API returns the full Application object after rollback
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRollbackResponse {
    /// Application after rollback
    pub application: Application,
}

/// Optimized summary for Rollback output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRollbackSummary {
    /// Application name
    pub name: String,
    /// History ID that was rolled back to
    pub rolled_back_to_id: i64,
    /// Whether the operation was a dry run
    pub dry_run: bool,
    /// Current sync status after rollback
    pub sync_status: Option<String>,
    /// Current sync revision after rollback
    pub sync_revision: Option<String>,
    /// Current health status after rollback
    pub health_status: Option<String>,
    /// Target revision after rollback
    pub target_revision: Option<String>,
    /// Whether prune was enabled
    pub prune_enabled: bool,
}

impl ApplicationRollbackSummary {
    pub fn from_application(app: Application, history_id: i64, dry_run: bool, prune: bool) -> Self {
        let name = app
            .metadata
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default();

        let sync_status = app
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone());

        let sync_revision = app
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sync| sync.revision.clone());

        let health_status = app
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|h| h.status.clone());

        let target_revision = app
            .spec
            .as_ref()
            .and_then(|s| s.source.as_ref())
            .and_then(|src| src.target_revision.clone());

        ApplicationRollbackSummary {
            name,
            rolled_back_to_id: history_id,
            dry_run,
            sync_status,
            sync_revision,
            health_status,
            target_revision,
            prune_enabled: prune,
        }
    }
}

/// SyncStrategy defines the sync strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStrategy {
    /// Apply strategy options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<SyncStrategyApply>,
    /// Hook strategy options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<SyncStrategyHook>,
}

/// SyncStrategyApply defines apply sync strategy options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStrategyApply {
    /// Force apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// SyncStrategyHook defines hook sync strategy options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStrategyHook {
    /// Force hook execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// SyncResource defines a specific resource to sync
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResource {
    /// Resource group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind
    pub kind: String,
    /// Resource name
    pub name: String,
    /// Resource namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// ApplicationSyncRequest is the request for syncing an application
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSyncRequest {
    /// Application name (required)
    pub name: String,
    /// Revision to sync to (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Dry run mode - if true, will not actually perform the sync
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Whether to prune resources that are no longer defined in Git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    /// Sync strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<SyncStrategy>,
    /// Specific resources to sync (if not specified, syncs all resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<SyncResource>>,
    /// Manifests to sync
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifests: Option<Vec<String>>,
    /// Sync options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_options: Option<Vec<String>>,
    /// Retry strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryStrategy>,
    /// Application namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

/// RetryStrategy defines the retry strategy for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryStrategy {
    /// Maximum number of retry attempts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Backoff strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<Backoff>,
}

/// Backoff defines the backoff strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backoff {
    /// Duration of the backoff (e.g., "5s", "1m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// Maximum duration of the backoff
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration: Option<String>,
    /// Factor to multiply the backoff duration by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factor: Option<i64>,
}

/// Optimized summary for Sync output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSyncSummary {
    /// Application name
    pub name: String,
    /// Whether the operation was a dry run
    pub dry_run: bool,
    /// Current sync status after sync
    pub sync_status: Option<String>,
    /// Current sync revision after sync
    pub sync_revision: Option<String>,
    /// Current health status after sync
    pub health_status: Option<String>,
    /// Target revision that was synced to
    pub target_revision: Option<String>,
    /// Whether prune was enabled
    pub prune_enabled: bool,
    /// Whether force was enabled
    pub force_enabled: bool,
    /// Sync options that were applied
    pub sync_options: Vec<String>,
    /// Number of resources synced (if resources were specified)
    pub resources_count: Option<usize>,
}

impl ApplicationSyncSummary {
    pub fn from_application(
        app: Application,
        dry_run: bool,
        prune: bool,
        force: bool,
        sync_options: Vec<String>,
        resources_count: Option<usize>,
    ) -> Self {
        let name = app
            .metadata
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default();

        let sync_status = app
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone());

        let sync_revision = app
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sync| sync.revision.clone());

        let health_status = app
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|h| h.status.clone());

        let target_revision = app
            .spec
            .as_ref()
            .and_then(|s| s.source.as_ref())
            .and_then(|src| src.target_revision.clone());

        ApplicationSyncSummary {
            name,
            dry_run,
            sync_status,
            sync_revision,
            health_status,
            target_revision,
            prune_enabled: prune,
            force_enabled: force,
            sync_options,
            resources_count,
        }
    }
}

/// ApplicationResourceResponse contains a single application resource manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationResourceResponse {
    /// The resource manifest as a string (YAML or JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
}

/// Optimized summary for GetResource output (context-efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationResourceSummary {
    /// Application name
    pub app_name: String,
    /// Resource kind (e.g., Pod, Service, Deployment)
    pub kind: String,
    /// Resource name
    pub resource_name: String,
    /// Resource namespace (if applicable)
    pub namespace: Option<String>,
    /// Resource API version
    pub version: String,
    /// Resource API group (empty string for core resources)
    pub group: Option<String>,
    /// Parsed manifest summary with key fields
    pub manifest_summary: ResourceManifestSummary,
    /// Raw manifest (truncated if too large)
    pub manifest: String,
}

/// Summary of key fields from a resource manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifestSummary {
    /// Resource API version from manifest
    pub api_version: Option<String>,
    /// Resource kind from manifest
    pub kind: Option<String>,
    /// Resource name from metadata
    pub name: Option<String>,
    /// Resource namespace from metadata
    pub namespace: Option<String>,
    /// Resource labels
    pub labels: Option<HashMap<String, String>>,
    /// Resource annotations (truncated if too many)
    pub annotations_count: Option<usize>,
    /// Creation timestamp
    pub creation_timestamp: Option<String>,
    /// Resource status summary (for resources with status)
    pub status_summary: Option<String>,
}

impl ApplicationResourceSummary {
    /// Create summary from manifest string
    pub fn from_manifest(
        app_name: String,
        kind: String,
        resource_name: String,
        namespace: Option<String>,
        version: String,
        group: Option<String>,
        manifest: String,
    ) -> Self {
        // Parse manifest to extract key fields
        let manifest_summary = Self::parse_manifest_summary(&manifest);

        // Truncate manifest if too large (keep first 10000 chars for context efficiency)
        let truncated_manifest = if manifest.len() > 10000 {
            format!(
                "{}... (truncated, {} total chars)",
                &manifest[..10000],
                manifest.len()
            )
        } else {
            manifest.clone()
        };

        ApplicationResourceSummary {
            app_name,
            kind,
            resource_name,
            namespace,
            version,
            group,
            manifest_summary,
            manifest: truncated_manifest,
        }
    }

    /// Parse manifest to extract key summary fields
    fn parse_manifest_summary(manifest: &str) -> ResourceManifestSummary {
        // Try to parse as YAML/JSON
        let parsed: Option<serde_json::Value> = serde_yaml::from_str(manifest)
            .ok()
            .or_else(|| serde_json::from_str(manifest).ok());

        if let Some(data) = parsed {
            let api_version = data["apiVersion"].as_str().map(|s| s.to_string());
            let kind = data["kind"].as_str().map(|s| s.to_string());

            let metadata = &data["metadata"];
            let name = metadata["name"].as_str().map(|s| s.to_string());
            let namespace = metadata["namespace"].as_str().map(|s| s.to_string());
            let creation_timestamp = metadata["creationTimestamp"]
                .as_str()
                .map(|s| s.to_string());

            let labels = metadata["labels"].as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            });

            let annotations_count = metadata["annotations"].as_object().map(|obj| obj.len());

            // Extract status summary if available
            let status_summary = if let Some(status) = data.get("status") {
                // Try to extract key status fields
                if let Some(phase) = status["phase"].as_str() {
                    Some(format!("Phase: {}", phase))
                } else if let Some(conditions) = status["conditions"].as_array() {
                    let ready = conditions
                        .iter()
                        .find(|c| c["type"].as_str() == Some("Ready"))
                        .and_then(|c| c["status"].as_str());
                    ready.map(|s| format!("Ready: {}", s))
                } else if let Some(replicas) = status["replicas"].as_i64() {
                    let ready_replicas = status["readyReplicas"].as_i64().unwrap_or(0);
                    Some(format!("{}/{} replicas ready", ready_replicas, replicas))
                } else {
                    Some("Status available".to_string())
                }
            } else {
                None
            };

            ResourceManifestSummary {
                api_version,
                kind,
                name,
                namespace,
                labels,
                annotations_count,
                creation_timestamp,
                status_summary,
            }
        } else {
            // Failed to parse, return empty summary
            ResourceManifestSummary {
                api_version: None,
                kind: None,
                name: None,
                namespace: None,
                labels: None,
                annotations_count: None,
                creation_timestamp: None,
                status_summary: None,
            }
        }
    }
}

/// RevisionHistory contains information about a deployment to the application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionHistory {
    /// ID is an auto incrementing identifier of the RevisionHistory
    pub id: i64,
    /// Revision holds the revision the sync was performed against
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// DeployedAt holds the time the sync operation completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed_at: Option<String>,
    /// DeployStartedAt holds the time the sync operation started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy_started_at: Option<String>,
    /// Source is a reference to the application source used for the sync operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ApplicationSource>,
    /// Sources is a reference to the application sources used for the sync operation (multi-source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<ApplicationSource>>,
    /// Revisions holds the revision of each source in sources field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revisions: Option<Vec<String>>,
    /// InitiatedBy contains information about who initiated the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiated_by: Option<OperationInitiator>,
}

/// OperationInitiator contains information about who started an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationInitiator {
    /// Username contains the name of a user who started operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Automated is set to true if operation was initiated automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automated: Option<bool>,
}

/// Optimized summary for application history (context efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationHistorySummary {
    /// Application name
    pub application_name: String,
    /// Total number of history entries
    pub total_entries: usize,
    /// History entries (sorted by ID descending - newest first)
    pub entries: Vec<RevisionHistorySummary>,
}

/// Optimized summary for a single revision history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionHistorySummary {
    /// History ID
    pub id: i64,
    /// Revision/commit hash (shortened for display)
    pub revision: String,
    /// Full revision (for programmatic use)
    pub revision_full: String,
    /// Deployed at timestamp
    pub deployed_at: String,
    /// Deploy duration (if deploy_started_at is available)
    pub deploy_duration: Option<String>,
    /// Source repository URL
    pub source_repo: Option<String>,
    /// Source path or chart
    pub source_path: Option<String>,
    /// Target revision (branch/tag)
    pub source_target_revision: Option<String>,
    /// Who initiated the deployment
    pub initiated_by: Option<String>,
    /// Whether deployment was automated
    pub automated: bool,
}

impl RevisionHistorySummary {
    /// Convert RevisionHistory to optimized summary
    pub fn from_revision_history(
        history: RevisionHistory,
    ) -> Self {
        let revision_full = history.revision.clone().unwrap_or_else(|| "unknown".to_string());
        let revision = if revision_full.len() > 8 {
            revision_full[..8].to_string()
        } else {
            revision_full.clone()
        };

        // Calculate deploy duration if both timestamps available
        let deploy_duration = if let (Some(started), Some(completed)) =
            (&history.deploy_started_at, &history.deployed_at) {
            // Simple duration string - could be enhanced with actual parsing
            Some(format!("from {} to {}", started, completed))
        } else {
            None
        };

        // Get source information (prefer single source, fallback to first of multi-source)
        let (source_repo, source_path, source_target_revision) = if let Some(source) = &history.source {
            (
                Some(source.repo_url.clone()),
                source.path.clone().or_else(|| source.chart.clone()),
                source.target_revision.clone(),
            )
        } else if let Some(sources) = &history.sources {
            if let Some(first_source) = sources.first() {
                (
                    Some(first_source.repo_url.clone()),
                    first_source.path.clone().or_else(|| first_source.chart.clone()),
                    first_source.target_revision.clone(),
                )
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        // Get initiator information
        let (initiated_by, automated) = if let Some(initiator) = &history.initiated_by {
            let username = if let Some(username) = &initiator.username {
                Some(username.clone())
            } else if initiator.automated.unwrap_or(false) {
                Some("Automated".to_string())
            } else {
                Some("Unknown".to_string())
            };
            (username, initiator.automated.unwrap_or(false))
        } else {
            (None, false)
        };

        RevisionHistorySummary {
            id: history.id,
            revision,
            revision_full,
            deployed_at: history.deployed_at.unwrap_or_else(|| "unknown".to_string()),
            deploy_duration,
            source_repo,
            source_path,
            source_target_revision,
            initiated_by,
            automated,
        }
    }
}

/// RefreshApplicationSummary contains the result of refreshing an application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshApplicationSummary {
    /// Application name
    pub application_name: String,
    /// Refresh type used
    pub refresh_type: String,
    /// Sync status before refresh
    pub sync_status_before: String,
    /// Sync status after refresh
    pub sync_status_after: String,
    /// Health status before refresh
    pub health_status_before: String,
    /// Health status after refresh
    pub health_status_after: String,
    /// Sync revision before refresh
    pub sync_revision_before: Option<String>,
    /// Sync revision after refresh
    pub sync_revision_after: Option<String>,
    /// Whether sync status changed
    pub sync_status_changed: bool,
    /// Whether health status changed
    pub health_status_changed: bool,
    /// Whether revision changed
    pub revision_changed: bool,
    /// Repository URL
    pub repo_url: Option<String>,
    /// Target revision (branch/tag)
    pub target_revision: Option<String>,
}

/// ApplicationResourcePatchRequest contains parameters for patching a resource
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationResourcePatchRequest {
    /// Application name (required)
    pub name: String,
    /// Patch content (required) - typically a JSON patch document
    pub patch: String,
    /// Resource namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Resource name (required)
    pub resource_name: String,
    /// Resource version (e.g., "v1")
    pub version: String,
    /// Resource API group (empty string for core resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Resource kind (e.g., "Deployment", "Service")
    pub kind: String,
    /// Patch type (e.g., "application/json-patch+json", "application/merge-patch+json", "application/strategic-merge-patch+json")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_type: Option<String>,
    /// Application namespace (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_namespace: Option<String>,
    /// Project identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}
