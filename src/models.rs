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
        let name = app.metadata.as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_default();

        let namespace = app.metadata.as_ref()
            .and_then(|m| m.namespace.clone());

        let project = app.spec.as_ref()
            .and_then(|s| s.project.clone());

        let repo_url = app.spec.as_ref()
            .and_then(|s| s.source.as_ref())
            .map(|src| src.repo_url.clone());

        let target_revision = app.spec.as_ref()
            .and_then(|s| s.source.as_ref())
            .and_then(|src| src.target_revision.clone());

        let destination_server = app.spec.as_ref()
            .and_then(|s| s.destination.as_ref())
            .and_then(|d| d.server.clone());

        let destination_namespace = app.spec.as_ref()
            .and_then(|s| s.destination.as_ref())
            .and_then(|d| d.namespace.clone());

        let sync_status = app.status.as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone());

        let health_status = app.status.as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|h| h.status.clone());

        let auto_sync = app.spec.as_ref()
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
        let sample_nodes: Vec<ResourceNodeSummary> = tree.nodes
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

        let name = metadata
            .map(|m| m.name.clone())
            .unwrap_or_default();

        let namespace = metadata
            .and_then(|m| m.namespace.clone());

        let labels = metadata
            .and_then(|m| m.labels.clone());

        let creation_timestamp = metadata
            .and_then(|m| m.creation_timestamp.clone());

        let project = spec
            .and_then(|s| s.project.clone());

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
        let potential_issue = is_error || is_warning
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
        let mut analyzed: Vec<AnalyzedLogEntry> = entries
            .into_iter()
            .map(AnalyzedLogEntry::from)
            .collect();

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
            *logs_by_level.entry(entry.level.as_str().to_string()).or_insert(0) += 1;
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
        let parsed: serde_json::Value = serde_yaml::from_str(yaml)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let kind = parsed["kind"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

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
            *manifests_by_kind
                .entry(manifest.kind.clone())
                .or_insert(0) += 1;
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
        let events: Vec<EventSummary> = event_list.items
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

        let involved_object_kind = event.involved_object.as_ref()
            .and_then(|obj| obj.kind.clone());
        let involved_object_name = event.involved_object.as_ref()
            .and_then(|obj| obj.name.clone());
        let involved_object_namespace = event.involved_object.as_ref()
            .and_then(|obj| obj.namespace.clone());

        let source_component = event.source.as_ref()
            .and_then(|src| src.component.clone());

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
        let message_short = metadata.message.as_ref().and_then(|msg| {
            msg.lines().next().map(|s| s.to_string())
        });

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
