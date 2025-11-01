mod argocd_client;
mod models;
mod tools;

use anyhow::{Context, Result};
use rmcp::ServiceExt;
use std::env;

use tools::ArgocdMcpHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logging to stderr
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting ArgoCD MCP Server");

    // Read environment variables
    let base_url = env::var("ARGOCD_BASE_URL")
        .context("ARGOCD_BASE_URL environment variable must be set")?;
    let access_token = env::var("ARGOCD_ACCESS_TOKEN")
        .context("ARGOCD_ACCESS_TOKEN environment variable must be set")?;

    tracing::info!("Connecting to ArgoCD at: {}", base_url);

    // Create handler and initialize with credentials
    let handler = ArgocdMcpHandler::new();
    handler.initialize(base_url, access_token).await
        .context("Failed to initialize ArgoCD client")?;

    tracing::info!("MCP Server initialized, starting stdio transport");

    // Start MCP server with stdio transport
    let service = handler.serve(rmcp::transport::stdio()).await.inspect_err(|e| {
        tracing::error!("Failed to serve: {:?}", e);
    })?;

    tracing::info!("MCP Server running");

    // Wait for completion
    service.waiting().await?;

    tracing::info!("MCP Server shutdown gracefully");
    Ok(())
}
