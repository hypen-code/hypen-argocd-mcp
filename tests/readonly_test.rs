use anyhow::Result;
use argocd_mcp_server::tools::ArgocdMcpHandler;
use serial_test::serial;

#[tokio::test]
async fn test_handler_default_not_read_only() -> Result<()> {
    let handler = ArgocdMcpHandler::new();
    assert!(!handler.is_read_only());
    Ok(())
}

#[tokio::test]
async fn test_handler_with_read_only_false() -> Result<()> {
    let handler = ArgocdMcpHandler::with_read_only(false);
    assert!(!handler.is_read_only());
    Ok(())
}

#[tokio::test]
async fn test_handler_with_read_only_true() -> Result<()> {
    let handler = ArgocdMcpHandler::with_read_only(true);
    assert!(handler.is_read_only());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_default() -> Result<()> {
    // Remove env var if it exists
    std::env::remove_var("ARGOCD_READ_ONLY");

    let handler = ArgocdMcpHandler::from_env();
    assert!(!handler.is_read_only());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_true() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "true");

    let handler = ArgocdMcpHandler::from_env();
    assert!(handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_false() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "false");

    let handler = ArgocdMcpHandler::from_env();
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_invalid() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "invalid");

    let handler = ArgocdMcpHandler::from_env();
    // Should default to false for invalid values
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_1() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "1");

    let handler = ArgocdMcpHandler::from_env();
    // "1" should parse as boolean but will fail, so defaults to false
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_yes() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "yes");

    let handler = ArgocdMcpHandler::from_env();
    // "yes" should fail to parse as boolean, defaults to false
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_case_sensitive() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "True");

    let handler = ArgocdMcpHandler::from_env();
    // Rust bool parsing is case-sensitive, so "True" fails to parse and defaults to false
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_handler_from_env_false_uppercase() -> Result<()> {
    std::env::set_var("ARGOCD_READ_ONLY", "FALSE");

    let handler = ArgocdMcpHandler::from_env();
    // Rust bool parsing is case-sensitive, so "FALSE" fails to parse and defaults to false
    assert!(!handler.is_read_only());

    // Cleanup
    std::env::remove_var("ARGOCD_READ_ONLY");
    Ok(())
}

#[tokio::test]
async fn test_read_only_server_info() -> Result<()> {
    use rmcp::ServerHandler;

    // Test read-only mode in server info
    let handler_ro = ArgocdMcpHandler::with_read_only(true);
    let info_ro = handler_ro.get_info();
    assert!(info_ro.instructions.unwrap().contains("READ-ONLY MODE"));

    // Test normal mode in server info
    let handler_normal = ArgocdMcpHandler::with_read_only(false);
    let info_normal = handler_normal.get_info();
    assert!(!info_normal.instructions.unwrap().contains("READ-ONLY MODE"));

    Ok(())
}

#[tokio::test]
async fn test_read_only_mode_mentions_env_var() -> Result<()> {
    use rmcp::ServerHandler;

    let handler = ArgocdMcpHandler::new();
    let info = handler.get_info();
    assert!(info.instructions.unwrap().contains("ARGOCD_READ_ONLY"));

    Ok(())
}

#[tokio::test]
async fn test_multiple_handlers_independent() -> Result<()> {
    let handler1 = ArgocdMcpHandler::with_read_only(true);
    let handler2 = ArgocdMcpHandler::with_read_only(false);

    assert!(handler1.is_read_only());
    assert!(!handler2.is_read_only());

    Ok(())
}

#[tokio::test]
async fn test_handler_clone_preserves_read_only() -> Result<()> {
    let handler1 = ArgocdMcpHandler::with_read_only(true);
    let handler2 = handler1.clone();

    assert!(handler1.is_read_only());
    assert!(handler2.is_read_only());

    Ok(())
}
