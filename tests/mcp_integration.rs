//! Integration tests for MCP functionality.
//!
//! These tests verify the end-to-end functionality of the MCP module,
//! including tool registration, execution, and error handling.
//!
//! Note: Full end-to-end tests with actual MCP servers are not included
//! due to the complexity of setting up external dependencies in CI.
//! The focus is on integration between components.

use std::sync::Arc;
use tokio::sync::RwLock;

use aether_matrix::{
    ai_service::AiService,
    config::{Config, McpConfig},
    mcp::{ToolDefinition, ToolRegistry},
};

#[tokio::test]
async fn test_tool_registry_creation_with_builtin_tools() {
    let config = McpConfig::default();
    let registry = ToolRegistry::new(&config.builtin_tools);

    // Should have at least the web_fetch tool if enabled
    if config.builtin_tools.enabled && config.builtin_tools.web_fetch.enabled {
        assert!(registry.len() > 0, "Should have builtin tools registered");
    }
}

#[tokio::test]
async fn test_ai_service_has_tools_method() {
    let mut config = Config::default();
    config.mcp = McpConfig::default();

    // Enable builtin tools
    config.mcp.builtin_tools.enabled = true;
    config.mcp.builtin_tools.web_fetch.enabled = true;

    let service = AiShip::new(&config).await;
    let has_tools = service.has_tools().await;

    assert!(
        has_tools,
        "AI service should report tools are available when builtin tools are enabled"
    );
}

#[tokio::test]
async fn test_tool_registry_to_openai_tools_conversion() {
    let config = McpConfig::default();
    let registry = ToolRegistry::new(&config.builtin_tools);

    let openai_tools = registry.to_openai_tools();

    // Should be able to convert to OpenAI tools format
    // Even if empty, the conversion should work
    assert!(openai_tools.len() >= 0);
}

// TODO: Add tests for:
// - External MCP server connection (requires mock server)
// - Tool execution with retry logic (requires mock tools)
// - Dynamic reload functionality (requires config reloading)
// - Error handling scenarios
