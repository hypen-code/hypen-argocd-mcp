use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use crate::models::{
    Application, ApplicationList, ApplicationSummaryOutput, ApplicationDetailOutput,
    ApplicationServerSideDiffResponse, ServerSideDiffSummary, ApplicationTree, ResourceTreeSummary,
    EventList, EventListSummary, LogEntry, PodLogsSummary, ManifestResponse, ManifestSummary,
    RevisionMetadata, RevisionMetadataSummary
};

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

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let app_list = response.json::<ApplicationList>().await
            .context("Failed to parse ApplicationList response")?;

        // Convert to optimized summaries
        let summaries = app_list.items
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app_list = response.json::<ApplicationList>().await
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

        tracing::debug!("Fetching application names from: {}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let app_list = response.json::<ApplicationList>().await
            .context("Failed to parse ApplicationList response")?;

        // Extract only names
        let names: Vec<String> = app_list.items
            .into_iter()
            .filter_map(|app| app.metadata.map(|m| m.name))
            .collect();

        Ok(names)
    }

    /// Perform server-side diff calculation using dry-run apply
    /// Returns optimized summaries to save context window
    pub async fn server_side_diff(
        &self,
        app_name: String,
        app_namespace: Option<String>,
        project: Option<String>,
        target_manifests: Option<Vec<String>>,
    ) -> Result<Vec<ServerSideDiffSummary>> {
        let mut url = format!("{}/api/v1/applications/{}/server-side-diff",
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
                params.push(format!("targetManifests={}", urlencoding::encode(&manifest)));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching server-side diff from: {}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let diff_response = response.json::<ApplicationServerSideDiffResponse>().await
            .context("Failed to parse ApplicationServerSideDiffResponse")?;

        // Convert to optimized summaries
        let summaries = diff_response.items
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
        let mut url = format!("{}/api/v1/applications/{}/server-side-diff",
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
                params.push(format!("targetManifests={}", urlencoding::encode(&manifest)));
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        tracing::debug!("Fetching server-side diff from: {}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let diff_response = response.json::<ApplicationServerSideDiffResponse>().await
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
        let mut url = format!("{}/api/v1/applications/{}/resource-tree",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let tree = response.json::<ApplicationTree>().await
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
        let mut url = format!("{}/api/v1/applications/{}/resource-tree",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let tree = response.json::<ApplicationTree>().await
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
        let mut url = format!("{}/api/v1/applications/{}",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let app = response.json::<Application>().await
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
        let mut url = format!("{}/api/v1/applications/{}",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let app = response.json::<Application>().await
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
        let mut url = format!("{}/api/v1/applications/{}/events",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let event_list = response.json::<EventList>().await
            .context("Failed to parse EventList response")?;

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
        let mut url = format!("{}/api/v1/applications/{}/events",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let event_list = response.json::<EventList>().await
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
        let mut url = format!("{}/api/v1/applications/{}/logs",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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
        let text = response.text().await
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
        let mut url = format!("{}/api/v1/applications/{}/manifests",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let manifest_response = response.json::<ManifestResponse>().await
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
        let mut url = format!("{}/api/v1/applications/{}/manifests",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let manifest_response = response.json::<ManifestResponse>().await
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
        let mut url = format!("{}/api/v1/applications/{}/revisions/{}/metadata",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
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

        let metadata = response.json::<RevisionMetadata>().await
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
        let mut url = format!("{}/api/v1/applications/{}/revisions/{}/metadata",
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

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to ArgoCD API")?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("ArgoCD API error ({}): {}", status, error_text);
        }

        let metadata = response.json::<RevisionMetadata>().await
            .context("Failed to parse RevisionMetadata response")?;

        Ok(metadata)
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
        let client = ArgocdClient::new(
            "".to_string(),
            "test-token".to_string(),
        );
        assert!(client.is_err());
    }

    #[test]
    fn test_client_empty_token() {
        let client = ArgocdClient::new(
            "https://argocd.example.com".to_string(),
            "".to_string(),
        );
        assert!(client.is_err());
    }
}
