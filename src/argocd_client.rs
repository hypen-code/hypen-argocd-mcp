use crate::models::{
    Application, ApplicationDetailOutput, ApplicationHistorySummary, ApplicationList,
    ApplicationResourceResponse, ApplicationResourceSummary, ApplicationRollbackSummary,
    ApplicationServerSideDiffResponse, ApplicationSummaryOutput, ApplicationSyncSummary,
    ApplicationSyncWindowsResponse, ApplicationSyncWindowsSummary, ApplicationTree, EventList,
    EventListSummary, LogEntry, ManifestResponse, ManifestSummary, PodLogsSummary,
    RefreshApplicationSummary, ResourceTreeSummary, RetryStrategy, RevisionHistorySummary,
    RevisionMetadata, RevisionMetadataSummary, ServerSideDiffSummary, SyncResource, SyncStrategy,
    SyncStrategyApply, SyncStrategyHook,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

/// ArgoCD API client with robust error handling
#[derive(Clone)]
pub struct ArgocdClient {
    base_url: String,
    access_token: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(default)]
    error: String,
    #[serde(default)]
    message: String,
}

impl ArgocdClient {
    /// Create a new ArgoCD client
    pub fn new(base_url: String, access_token: String) -> Result<Self> {
        // Validate inputs
        if base_url.is_empty() {
            anyhow::bail!("base_url cannot be empty");
        }
        if access_token.is_empty() {
            anyhow::bail!("access_token cannot be empty");
        }

        // Check if we should skip TLS verification (useful for self-signed certs)
        let insecure_env = std::env::var("ARGOCD_INSECURE").unwrap_or_else(|_| "false".to_string());
        let skip_tls_verify = insecure_env.to_lowercase() == "true";

        tracing::info!("ARGOCD_INSECURE environment variable: {:?}", insecure_env);
        tracing::info!("Skip TLS verification: {}", skip_tls_verify);

        let mut client_builder = Client::builder().timeout(std::time::Duration::from_secs(30));

        if skip_tls_verify {
            tracing::warn!("TLS certificate verification is DISABLED (ARGOCD_INSECURE=true)");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        } else {
            tracing::info!("TLS certificate verification is ENABLED");
        }

        let client = client_builder
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            access_token,
            client,
        })
    }

    /// List applications with optional filters
    /// Returns optimized summaries to save context window
    pub async fn list_applications(
        &self,
        name: Option<String>,
        projects: Option<Vec<String>>,
        selector: Option<String>,
        repo: Option<String>,
        app_namespace: Option<String>,
    ) -> Result<Vec<ApplicationSummaryOutput>> {
        let mut url = format!("{}/api/v1/applications", self.base_url);
        let mut params = Vec::new();

        if let Some(n) = name {
            params.push(format!("name={}", urlencoding::encode(&n)));
        }
        if let Some(projs) = projects {
            for proj in projs {
                params.push(format!("projects={}", urlencoding::encode(&proj)));
            }
        }
        if let Some(sel) = selector {
            params.push(format!("selector={}", urlencoding::encode(&sel)));
        }
        if let Some(r) = repo {
            params.push(format!("repo={}", urlencoding::encode(&r)));
        }
        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching applications from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let app_list = response
            .json::<ApplicationList>()
            .await
            .context("Failed to parse ApplicationList response")?;

        // Convert to optimized summaries
        let summaries = app_list
            .items
            .into_iter()
            .map(ApplicationSummaryOutput::from)
            .collect();

        Ok(summaries)
    }

    /// Get full application details (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn list_applications_full(
        &self,
        name: Option<String>,
        projects: Option<Vec<String>>,
        selector: Option<String>,
        repo: Option<String>,
        app_namespace: Option<String>,
    ) -> Result<ApplicationList> {
        let mut url = format!("{}/api/v1/applications", self.base_url);
        let mut params = Vec::new();

        if let Some(n) = name {
            params.push(format!("name={}", urlencoding::encode(&n)));
        }
        if let Some(projs) = projects {
            for proj in projs {
                params.push(format!("projects={}", urlencoding::encode(&proj)));
            }
        }
        if let Some(sel) = selector {
            params.push(format!("selector={}", urlencoding::encode(&sel)));
        }
        if let Some(r) = repo {
            params.push(format!("repo={}", urlencoding::encode(&r)));
        }
        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching applications from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app_list = response
            .json::<ApplicationList>()
            .await
            .context("Failed to parse ApplicationList response")?;

        Ok(app_list)
    }

    /// List only application names (minimal response for name lookup and typo correction)
    /// This is highly optimized for context window efficiency
    pub async fn list_application_names(
        &self,
        projects: Option<Vec<String>>,
        selector: Option<String>,
        repo: Option<String>,
        app_namespace: Option<String>,
    ) -> Result<Vec<String>> {
        let mut url = format!("{}/api/v1/applications", self.base_url);
        let mut params = Vec::new();

        if let Some(projs) = projects {
            for proj in projs {
                params.push(format!("projects={}", urlencoding::encode(&proj)));
            }
        }
        if let Some(sel) = selector {
            params.push(format!("selector={}", urlencoding::encode(&sel)));
        }
        if let Some(r) = repo {
            params.push(format!("repo={}", urlencoding::encode(&r)));
        }
        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::info!("Fetching application names from: {}", url);
        tracing::info!(
            "ARGOCD_INSECURE env var: {:?}",
            std::env::var("ARGOCD_INSECURE")
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to send request to ArgoCD API ({}): {}", url, e)
            })?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let app_list = response
            .json::<ApplicationList>()
            .await
            .context("Failed to parse ApplicationList response")?;

        // Extract only names
        let names: Vec<String> = app_list
            .items
            .into_iter()
            .filter_map(|app| app.metadata.map(|m| m.name))
            .collect();

        Ok(names)
    }

    /// Perform server-side diff calculation using dry-run apply
    /// Returns optimized summaries to save context window
    ///
    /// **Note**: This endpoint may not be available in all ArgoCD versions.
    /// If you receive a 404 error, your ArgoCD instance may not support this feature.
    /// This feature typically requires ArgoCD v2.5+ with Server-Side Apply support.
    pub async fn server_side_diff(
        &self,
        app_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
        target_manifests: Option<Vec<String>>,
    ) -> Result<Vec<ServerSideDiffSummary>> {
        let mut url = format!(
            "{}/api/v1/applications/{}/server-side-diff",
            self.base_url,
            urlencoding::encode(&app_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }
        if let Some(proj) = project {
            params.push(format!("project={}", urlencoding::encode(&proj)));
        }
        if let Some(manifests) = target_manifests {
            for manifest in manifests {
                params.push(format!(
                    "targetManifests={}",
                    urlencoding::encode(&manifest)
                ));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching server-side diff from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let diff_response = response
            .json::<ApplicationServerSideDiffResponse>()
            .await
            .context("Failed to parse ApplicationServerSideDiffResponse")?;

        // Convert to optimized summaries
        let summaries = diff_response
            .items
            .into_iter()
            .map(ServerSideDiffSummary::from)
            .collect();

        Ok(summaries)
    }

    /// Get full server-side diff details (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn server_side_diff_full(
        &self,
        app_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
        target_manifests: Option<Vec<String>>,
    ) -> Result<ApplicationServerSideDiffResponse> {
        let mut url = format!(
            "{}/api/v1/applications/{}/server-side-diff",
            self.base_url,
            urlencoding::encode(&app_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }
        if let Some(proj) = project {
            params.push(format!("project={}", urlencoding::encode(&proj)));
        }
        if let Some(manifests) = target_manifests {
            for manifest in manifests {
                params.push(format!(
                    "targetManifests={}",
                    urlencoding::encode(&manifest)
                ));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching server-side diff from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let diff_response = response
            .json::<ApplicationServerSideDiffResponse>()
            .await
            .context("Failed to parse ApplicationServerSideDiffResponse")?;

        Ok(diff_response)
    }

    /// Get resource tree for an application
    /// Returns optimized summary to save context window
    pub async fn resource_tree(
        &self,
        application_name: String,
        namespace: Option<String>,
        name: Option<String>,
        version: Option<String>,
        group: Option<String>,
        kind: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ResourceTreeSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource-tree",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = namespace {
            params.push(format!("namespace={}", urlencoding::encode(&ns)));
        }
        if let Some(n) = name {
            params.push(format!("name={}", urlencoding::encode(&n)));
        }
        if let Some(v) = version {
            params.push(format!("version={}", urlencoding::encode(&v)));
        }
        if let Some(g) = group {
            params.push(format!("group={}", urlencoding::encode(&g)));
        }
        if let Some(k) = kind {
            params.push(format!("kind={}", urlencoding::encode(&k)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource tree from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let tree = response
            .json::<ApplicationTree>()
            .await
            .context("Failed to parse ApplicationTree response")?;

        // Convert to optimized summary
        Ok(ResourceTreeSummary::from(tree))
    }

    /// Get full resource tree for an application (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn resource_tree_full(
        &self,
        application_name: String,
        namespace: Option<String>,
        name: Option<String>,
        version: Option<String>,
        group: Option<String>,
        kind: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationTree> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource-tree",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = namespace {
            params.push(format!("namespace={}", urlencoding::encode(&ns)));
        }
        if let Some(n) = name {
            params.push(format!("name={}", urlencoding::encode(&n)));
        }
        if let Some(v) = version {
            params.push(format!("version={}", urlencoding::encode(&v)));
        }
        if let Some(g) = group {
            params.push(format!("group={}", urlencoding::encode(&g)));
        }
        if let Some(k) = kind {
            params.push(format!("kind={}", urlencoding::encode(&k)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource tree from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let tree = response
            .json::<ApplicationTree>()
            .await
            .context("Failed to parse ApplicationTree response")?;

        Ok(tree)
    }

    /// Get a single application by name
    /// Returns optimized detail output to save context window
    pub async fn get_application(
        &self,
        name: String,
        app_namespace: Option<String>,
        project: Option<String>,
        refresh: Option<String>,
        resource_version: Option<String>,
    ) -> Result<ApplicationDetailOutput> {
        let mut url = format!(
            "{}/api/v1/applications/{}",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }
        if let Some(proj) = project {
            params.push(format!("project={}", urlencoding::encode(&proj)));
        }
        if let Some(ref_type) = refresh {
            params.push(format!("refresh={}", urlencoding::encode(&ref_type)));
        }
        if let Some(rv) = resource_version {
            params.push(format!("resourceVersion={}", urlencoding::encode(&rv)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching application from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        // Convert to optimized detail output
        Ok(ApplicationDetailOutput::from(app))
    }

    /// Get full application details (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn get_application_full(
        &self,
        name: String,
        app_namespace: Option<String>,
        project: Option<String>,
        refresh: Option<String>,
        resource_version: Option<String>,
    ) -> Result<Application> {
        let mut url = format!(
            "{}/api/v1/applications/{}",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ns) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ns)));
        }
        if let Some(proj) = project {
            params.push(format!("project={}", urlencoding::encode(&proj)));
        }
        if let Some(ref_type) = refresh {
            params.push(format!("refresh={}", urlencoding::encode(&ref_type)));
        }
        if let Some(rv) = resource_version {
            params.push(format!("resourceVersion={}", urlencoding::encode(&rv)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching application from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        Ok(app)
    }

    /// List resource events for an application or specific resource
    /// Returns optimized summary to save context window
    pub async fn list_resource_events(
        &self,
        application_name: String,
        resource_namespace: Option<String>,
        resource_name: Option<String>,
        resource_uid: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<EventListSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/events",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(rns) = resource_namespace {
            params.push(format!("resourceNamespace={}", urlencoding::encode(&rns)));
        }
        if let Some(rn) = resource_name {
            params.push(format!("resourceName={}", urlencoding::encode(&rn)));
        }
        if let Some(ruid) = resource_uid {
            params.push(format!("resourceUID={}", urlencoding::encode(&ruid)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource events from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        // Get the response text first for better error handling
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Try to parse the response
        let event_list: EventList = match serde_json::from_str(&response_text) {
            Ok(list) => list,
            Err(e) => {
                // Log the actual response for debugging
                tracing::error!(
                    "Failed to parse EventList response. Error: {}, Response body: {}",
                    e,
                    response_text
                );
                // Return empty event list if parsing fails
                EventList {
                    metadata: None,
                    items: Vec::new(),
                }
            }
        };

        // Convert to optimized summary
        Ok(EventListSummary::from(event_list))
    }

    /// Get full event list (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn list_resource_events_full(
        &self,
        application_name: String,
        resource_namespace: Option<String>,
        resource_name: Option<String>,
        resource_uid: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<EventList> {
        let mut url = format!(
            "{}/api/v1/applications/{}/events",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(rns) = resource_namespace {
            params.push(format!("resourceNamespace={}", urlencoding::encode(&rns)));
        }
        if let Some(rn) = resource_name {
            params.push(format!("resourceName={}", urlencoding::encode(&rn)));
        }
        if let Some(ruid) = resource_uid {
            params.push(format!("resourceUID={}", urlencoding::encode(&ruid)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource events from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let event_list = response
            .json::<EventList>()
            .await
            .context("Failed to parse EventList response")?;

        Ok(event_list)
    }

    /// Get pod logs for an application resource
    /// Returns optimized summary with log analysis
    pub async fn pod_logs(
        &self,
        application_name: String,
        namespace: Option<String>,
        pod_name: Option<String>,
        container: Option<String>,
        since_seconds: Option<i64>,
        tail_lines: Option<i64>,
        previous: Option<bool>,
        filter: Option<String>,
        kind: Option<String>,
        group: Option<String>,
        resource_name: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
        filter_errors_only: bool,
    ) -> Result<PodLogsSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/logs",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = namespace {
            params.push(format!("namespace={}", urlencoding::encode(&ns)));
        }
        if let Some(pn) = &pod_name {
            params.push(format!("podName={}", urlencoding::encode(pn)));
        }
        if let Some(c) = &container {
            params.push(format!("container={}", urlencoding::encode(c)));
        }
        if let Some(ss) = since_seconds {
            params.push(format!("sinceSeconds={}", ss));
        }
        if let Some(tl) = tail_lines {
            params.push(format!("tailLines={}", tl));
        }
        if let Some(true) = previous {
            params.push("previous=true".to_string());
        }
        if let Some(f) = filter {
            params.push(format!("filter={}", urlencoding::encode(&f)));
        }
        if let Some(k) = kind {
            params.push(format!("kind={}", urlencoding::encode(&k)));
        }
        if let Some(g) = group {
            params.push(format!("group={}", urlencoding::encode(&g)));
        }
        if let Some(rn) = resource_name {
            params.push(format!("resourceName={}", urlencoding::encode(&rn)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        // Disable follow for non-streaming response
        params.push("follow=false".to_string());

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching pod logs from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        // The logs endpoint returns newline-delimited JSON (NDJSON/JSON streaming)
        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Parse NDJSON - each line is a JSON object with either "result" or "error"
        let mut log_entries = Vec::new();
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as streaming response wrapper
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                // Check if it has a "result" field (log entry) or "error" field
                if let Some(result) = parsed.get("result") {
                    if let Ok(entry) = serde_json::from_value::<LogEntry>(result.clone()) {
                        log_entries.push(entry);
                    }
                } else if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                    // Direct log entry format
                    log_entries.push(entry);
                }
            }
        }

        // Convert to optimized summary with analysis
        Ok(PodLogsSummary::from_entries(
            log_entries,
            pod_name,
            container,
            tail_lines,
            filter_errors_only,
        ))
    }

    /// Get application manifests
    /// Returns optimized summary with parsed manifests
    pub async fn get_manifests(
        &self,
        application_name: String,
        revision: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
        source_positions: Option<Vec<i64>>,
        revisions: Option<Vec<String>>,
    ) -> Result<ManifestSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/manifests",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(rev) = revision {
            params.push(format!("revision={}", urlencoding::encode(&rev)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }
        if let Some(positions) = source_positions {
            for pos in positions {
                params.push(format!("sourcePositions={}", pos));
            }
        }
        if let Some(revs) = revisions {
            for rev in revs {
                params.push(format!("revisions={}", urlencoding::encode(&rev)));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching manifests from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let manifest_response = response
            .json::<ManifestResponse>()
            .await
            .context("Failed to parse ManifestResponse")?;

        // Convert to optimized summary
        Ok(ManifestSummary::from(manifest_response))
    }

    /// Get full manifest response (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn get_manifests_full(
        &self,
        application_name: String,
        revision: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
        source_positions: Option<Vec<i64>>,
        revisions: Option<Vec<String>>,
    ) -> Result<ManifestResponse> {
        let mut url = format!(
            "{}/api/v1/applications/{}/manifests",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(rev) = revision {
            params.push(format!("revision={}", urlencoding::encode(&rev)));
        }
        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }
        if let Some(positions) = source_positions {
            for pos in positions {
                params.push(format!("sourcePositions={}", pos));
            }
        }
        if let Some(revs) = revisions {
            for rev in revs {
                params.push(format!("revisions={}", urlencoding::encode(&rev)));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching manifests from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let manifest_response = response
            .json::<ManifestResponse>()
            .await
            .context("Failed to parse ManifestResponse")?;

        Ok(manifest_response)
    }

    /// Get revision metadata for a specific revision of an application
    /// Returns optimized summary with revision information
    pub async fn revision_metadata(
        &self,
        application_name: String,
        revision: String,
        app_namespace: Option<String>,
        project: Option<String>,
        source_index: Option<i32>,
        version_id: Option<i32>,
    ) -> Result<RevisionMetadataSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/revisions/{}/metadata",
            self.base_url,
            urlencoding::encode(&application_name),
            urlencoding::encode(&revision)
        );
        let mut params = Vec::new();

        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }
        if let Some(si) = source_index {
            params.push(format!("sourceIndex={}", si));
        }
        if let Some(vid) = version_id {
            params.push(format!("versionId={}", vid));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching revision metadata from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let metadata = response
            .json::<RevisionMetadata>()
            .await
            .context("Failed to parse RevisionMetadata response")?;

        // Convert to optimized summary
        Ok(RevisionMetadataSummary::from(metadata))
    }

    /// Get full revision metadata (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn revision_metadata_full(
        &self,
        application_name: String,
        revision: String,
        app_namespace: Option<String>,
        project: Option<String>,
        source_index: Option<i32>,
        version_id: Option<i32>,
    ) -> Result<RevisionMetadata> {
        let mut url = format!(
            "{}/api/v1/applications/{}/revisions/{}/metadata",
            self.base_url,
            urlencoding::encode(&application_name),
            urlencoding::encode(&revision)
        );
        let mut params = Vec::new();

        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }
        if let Some(si) = source_index {
            params.push(format!("sourceIndex={}", si));
        }
        if let Some(vid) = version_id {
            params.push(format!("versionId={}", vid));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching revision metadata from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let metadata = response
            .json::<RevisionMetadata>()
            .await
            .context("Failed to parse RevisionMetadata response")?;

        Ok(metadata)
    }

    /// Get application sync windows
    /// Returns optimized summary with sync window information
    ///
    /// **Note**: This endpoint may not be available in all ArgoCD versions.
    /// If you receive a 404 error, your ArgoCD instance may not support application-level
    /// sync windows, or sync windows may need to be configured at the project level.
    /// This feature typically requires ArgoCD v2.6+.
    pub async fn get_application_sync_windows(
        &self,
        application_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationSyncWindowsSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/sync-windows",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching application sync windows from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let sync_windows_response = response
            .json::<ApplicationSyncWindowsResponse>()
            .await
            .context("Failed to parse ApplicationSyncWindowsResponse")?;

        // Convert to optimized summary
        Ok(ApplicationSyncWindowsSummary::from(sync_windows_response))
    }

    /// Get full application sync windows (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn get_application_sync_windows_full(
        &self,
        application_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationSyncWindowsResponse> {
        let mut url = format!(
            "{}/api/v1/applications/{}/sync-windows",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ans) = app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(&ans)));
        }
        if let Some(p) = project {
            params.push(format!("project={}", urlencoding::encode(&p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching application sync windows from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let sync_windows_response = response
            .json::<ApplicationSyncWindowsResponse>()
            .await
            .context("Failed to parse ApplicationSyncWindowsResponse")?;

        Ok(sync_windows_response)
    }

    /// Rollback an application to a previous deployed version by History ID
    /// Returns optimized summary to save context window
    pub async fn rollback_application(
        &self,
        name: String,
        id: i64,
        dry_run: Option<bool>,
        prune: Option<bool>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationRollbackSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/rollback",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Rolling back application at: {}", url);

        // Build request body
        let request_body = serde_json::json!({
            "name": name,
            "id": id,
            "dryRun": dry_run.unwrap_or(false),
            "prune": prune.unwrap_or(false),
            "appNamespace": app_namespace,
            "project": project,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        // Convert to optimized summary
        Ok(ApplicationRollbackSummary::from_application(
            app,
            id,
            dry_run.unwrap_or(false),
            prune.unwrap_or(false),
        ))
    }

    /// Rollback an application (returns full Application object)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn rollback_application_full(
        &self,
        name: String,
        id: i64,
        dry_run: Option<bool>,
        prune: Option<bool>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<Application> {
        let mut url = format!(
            "{}/api/v1/applications/{}/rollback",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Rolling back application at: {}", url);

        // Build request body
        let request_body = serde_json::json!({
            "name": name,
            "id": id,
            "dryRun": dry_run.unwrap_or(false),
            "prune": prune.unwrap_or(false),
            "appNamespace": app_namespace,
            "project": project,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        Ok(app)
    }

    /// Sync an application to its target state
    /// Returns optimized summary to save context window
    #[allow(clippy::too_many_arguments)]
    pub async fn sync_application(
        &self,
        name: String,
        revision: Option<String>,
        dry_run: Option<bool>,
        prune: Option<bool>,
        force: Option<bool>,
        resources: Option<Vec<SyncResource>>,
        sync_options: Option<Vec<String>>,
        retry: Option<RetryStrategy>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationSyncSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/sync",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Syncing application at: {}", url);

        // Build strategy
        let strategy = if let Some(true) = force {
            Some(SyncStrategy {
                apply: Some(SyncStrategyApply { force: Some(true) }),
                hook: Some(SyncStrategyHook { force: Some(true) }),
            })
        } else {
            None
        };

        // Build request body
        let request_body = serde_json::json!({
            "name": name,
            "revision": revision,
            "dryRun": dry_run.unwrap_or(false),
            "prune": prune.unwrap_or(false),
            "strategy": strategy,
            "resources": resources,
            "syncOptions": sync_options,
            "retry": retry,
            "appNamespace": app_namespace,
            "project": project,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        // Convert to optimized summary
        Ok(ApplicationSyncSummary::from_application(
            app,
            dry_run.unwrap_or(false),
            prune.unwrap_or(false),
            force.unwrap_or(false),
            sync_options.unwrap_or_default(),
            resources.as_ref().map(|r| r.len()),
        ))
    }

    /// Sync an application (returns full Application object)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub async fn sync_application_full(
        &self,
        name: String,
        revision: Option<String>,
        dry_run: Option<bool>,
        prune: Option<bool>,
        force: Option<bool>,
        resources: Option<Vec<SyncResource>>,
        sync_options: Option<Vec<String>>,
        retry: Option<RetryStrategy>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<Application> {
        let mut url = format!(
            "{}/api/v1/applications/{}/sync",
            self.base_url,
            urlencoding::encode(&name)
        );
        let mut params = Vec::new();

        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Syncing application at: {}", url);

        // Build strategy
        let strategy = if let Some(true) = force {
            Some(SyncStrategy {
                apply: Some(SyncStrategyApply { force: Some(true) }),
                hook: Some(SyncStrategyHook { force: Some(true) }),
            })
        } else {
            None
        };

        // Build request body
        let request_body = serde_json::json!({
            "name": name,
            "revision": revision,
            "dryRun": dry_run.unwrap_or(false),
            "prune": prune.unwrap_or(false),
            "strategy": strategy,
            "resources": resources,
            "syncOptions": sync_options,
            "retry": retry,
            "appNamespace": app_namespace,
            "project": project,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app = response
            .json::<Application>()
            .await
            .context("Failed to parse Application response")?;

        Ok(app)
    }

    /// Get a specific resource from an ArgoCD application
    /// Returns optimized summary to save context window
    pub async fn get_resource(
        &self,
        application_name: String,
        namespace: Option<String>,
        resource_name: String,
        version: String,
        group: Option<String>,
        kind: String,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationResourceSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = &namespace {
            params.push(format!("namespace={}", urlencoding::encode(ns)));
        }
        params.push(format!("resourceName={}", urlencoding::encode(&resource_name)));
        params.push(format!("version={}", urlencoding::encode(&version)));
        if let Some(g) = &group {
            params.push(format!("group={}", urlencoding::encode(g)));
        }
        params.push(format!("kind={}", urlencoding::encode(&kind)));
        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let resource_response = response
            .json::<ApplicationResourceResponse>()
            .await
            .context("Failed to parse ApplicationResourceResponse")?;

        // Convert to optimized summary
        let manifest = resource_response.manifest.unwrap_or_default();
        Ok(ApplicationResourceSummary::from_manifest(
            application_name,
            kind,
            resource_name,
            namespace,
            version,
            group,
            manifest,
        ))
    }

    /// Get full resource response (not optimized)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    pub async fn get_resource_full(
        &self,
        application_name: String,
        namespace: Option<String>,
        resource_name: String,
        version: String,
        group: Option<String>,
        kind: String,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationResourceResponse> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = &namespace {
            params.push(format!("namespace={}", urlencoding::encode(ns)));
        }
        params.push(format!("resourceName={}", urlencoding::encode(&resource_name)));
        params.push(format!("version={}", urlencoding::encode(&version)));
        if let Some(g) = &group {
            params.push(format!("group={}", urlencoding::encode(g)));
        }
        params.push(format!("kind={}", urlencoding::encode(&kind)));
        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching resource from: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let resource_response = response
            .json::<ApplicationResourceResponse>()
            .await
            .context("Failed to parse ApplicationResourceResponse")?;

        Ok(resource_response)
    }

    /// Patch a specific resource in an ArgoCD application
    /// Returns optimized summary to save context window
    #[allow(clippy::too_many_arguments)]
    pub async fn patch_resource(
        &self,
        application_name: String,
        namespace: Option<String>,
        resource_name: String,
        version: String,
        group: Option<String>,
        kind: String,
        patch: String,
        patch_type: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationResourceSummary> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = &namespace {
            params.push(format!("namespace={}", urlencoding::encode(ns)));
        }
        params.push(format!("resourceName={}", urlencoding::encode(&resource_name)));
        params.push(format!("version={}", urlencoding::encode(&version)));
        if let Some(g) = &group {
            params.push(format!("group={}", urlencoding::encode(g)));
        }
        params.push(format!("kind={}", urlencoding::encode(&kind)));
        if let Some(pt) = &patch_type {
            params.push(format!("patchType={}", urlencoding::encode(pt)));
        }
        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Patching resource at: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(patch.clone())
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as JSON error
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = if !err.message.is_empty() {
                    err.message
                } else if !err.error.is_empty() {
                    err.error
                } else {
                    error_text
                };
                anyhow::bail!("ArgoCD API error ({}): {}", status, msg);
            } else {
                anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
            }
        }

        let resource_response = response
            .json::<ApplicationResourceResponse>()
            .await
            .context("Failed to parse ApplicationResourceResponse")?;

        // Convert to optimized summary
        let manifest = resource_response.manifest.unwrap_or_default();
        Ok(ApplicationResourceSummary::from_manifest(
            application_name,
            kind,
            resource_name,
            namespace,
            version,
            group,
            manifest,
        ))
    }

    /// Patch resource (returns full ApplicationResourceResponse)
    /// This method is part of the public API and used in tests
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub async fn patch_resource_full(
        &self,
        application_name: String,
        namespace: Option<String>,
        resource_name: String,
        version: String,
        group: Option<String>,
        kind: String,
        patch: String,
        patch_type: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationResourceResponse> {
        let mut url = format!(
            "{}/api/v1/applications/{}/resource",
            self.base_url,
            urlencoding::encode(&application_name)
        );
        let mut params = Vec::new();

        if let Some(ns) = &namespace {
            params.push(format!("namespace={}", urlencoding::encode(ns)));
        }
        params.push(format!("resourceName={}", urlencoding::encode(&resource_name)));
        params.push(format!("version={}", urlencoding::encode(&version)));
        if let Some(g) = &group {
            params.push(format!("group={}", urlencoding::encode(g)));
        }
        params.push(format!("kind={}", urlencoding::encode(&kind)));
        if let Some(pt) = &patch_type {
            params.push(format!("patchType={}", urlencoding::encode(pt)));
        }
        if let Some(ans) = &app_namespace {
            params.push(format!("appNamespace={}", urlencoding::encode(ans)));
        }
        if let Some(p) = &project {
            params.push(format!("project={}", urlencoding::encode(p)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Patching resource at: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .body(patch)
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let resource_response = response
            .json::<ApplicationResourceResponse>()
            .await
            .context("Failed to parse ApplicationResourceResponse")?;

        Ok(resource_response)
    }

    /// Get application deployment history
    /// Returns optimized summary to save context window
    pub async fn get_application_history(
        &self,
        application_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<ApplicationHistorySummary> {
        // Get the full application to access history
        let app = self
            .get_application_full(application_name.clone(), app_namespace, project, None, None)
            .await?;

        // Extract history from application status
        let history = app
            .status
            .and_then(|status| status.history)
            .unwrap_or_default();

        // Convert to optimized summaries and sort by ID descending (newest first)
        let mut entries: Vec<RevisionHistorySummary> = history
            .into_iter()
            .map(RevisionHistorySummary::from_revision_history)
            .collect();

        // Sort by ID descending (newest first)
        entries.sort_by(|a, b| b.id.cmp(&a.id));

        let total_entries = entries.len();

        Ok(ApplicationHistorySummary {
            application_name,
            total_entries,
            entries,
        })
    }

    /// Refresh an application from Git repository
    /// Returns summary showing before/after state and what changed
    pub async fn refresh_application(
        &self,
        application_name: String,
        refresh_type: Option<String>,
        app_namespace: Option<String>,
        project: Option<String>,
    ) -> Result<RefreshApplicationSummary> {
        // Get application state BEFORE refresh
        let app_before = self
            .get_application_full(
                application_name.clone(),
                app_namespace.clone(),
                project.clone(),
                None,
                None,
            )
            .await?;

        // Extract before state
        let sync_status_before = app_before
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let health_status_before = app_before
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|health| health.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let sync_revision_before = app_before
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sync| sync.revision.clone());

        // Get application WITH refresh parameter
        let refresh_param = refresh_type.clone().or_else(|| Some("hard".to_string()));
        let app_after = self
            .get_application_full(
                application_name.clone(),
                app_namespace.clone(),
                project.clone(),
                refresh_param.clone(),
                None,
            )
            .await?;

        // Extract after state
        let sync_status_after = app_after
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .map(|sync| sync.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let health_status_after = app_after
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .map(|health| health.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let sync_revision_after = app_after
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sync| sync.revision.clone());

        // Extract source info
        let (repo_url, target_revision) = if let Some(spec) = &app_after.spec {
            if let Some(source) = &spec.source {
                (Some(source.repo_url.clone()), source.target_revision.clone())
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Calculate what changed
        let sync_status_changed = sync_status_before != sync_status_after;
        let health_status_changed = health_status_before != health_status_after;
        let revision_changed = sync_revision_before != sync_revision_after;

        Ok(RefreshApplicationSummary {
            application_name,
            refresh_type: refresh_param.unwrap_or_else(|| "hard".to_string()),
            sync_status_before,
            sync_status_after,
            health_status_before,
            health_status_after,
            sync_revision_before,
            sync_revision_after,
            sync_status_changed,
            health_status_changed,
            revision_changed,
            repo_url: repo_url.or_else(|| Some("Unknown".to_string())),
            target_revision,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ArgocdClient::new(
            "https://argocd.example.com".to_string(),
            "test-token".to_string(),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_empty_url() {
        let client = ArgocdClient::new("".to_string(), "test-token".to_string());
        assert!(client.is_err());
    }

    #[test]
    fn test_client_empty_token() {
        let client = ArgocdClient::new("https://argocd.example.com".to_string(), "".to_string());
        assert!(client.is_err());
    }
}
