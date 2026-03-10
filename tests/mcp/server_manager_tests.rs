//! Unit tests for MCP server manager functionality.
//!
//! These tests focus on the logic of server management, status tracking,
//! retry logic, and configuration handling without requiring actual
//! external MCP servers to be running.

use aether_matrix::mcp::{
    config::{ExternalServerConfig, McpConfig, TransportType},
    server_manager::{McpServer, McpServerManager, ServerStatus},
    tool_registry::ToolRegistry,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test McpServer constructor creates server with correct initial state.
#[tokio::test]
async fn test_mcp_server_constructor() {
    let config = ExternalServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("echo".to_string()),
        args: Some(vec!["hello".to_string()]),
        url: None,
    };

    let server = McpServer::new(config.clone());
    
    assert_eq!(server.config.name, "test-server");
    assert_eq!(server.status, ServerStatus::Disconnected);
    assert_eq!(server.retry_count, 0);
    assert!(server.last_retry.is_none());
    assert!(server.peer.is_none());
}

/// Test McpServer status management and transitions.
#[tokio::test]
async fn test_mcp_server_status_transitions() {
    let config = ExternalServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("echo".to_string()),
        args: Some(vec!["hello".to_string()]),
        url: None,
    };

    let mut server = McpServer::new(config);
    
    // Initial state
    assert_eq!(server.status(), &ServerStatus::Disconnected);
    
    // Test should_retry when disconnected and enabled
    assert!(server.should_retry());
    
    // Test disabled server should not retry
    server.config.enabled = false;
    assert!(!server.should_retry());
    server.config.enabled = true; // Reset for other tests
    
    // Test that connecting sets status correctly (even if it fails)
    // Note: This will fail because echo isn't an MCP server, but we can test status transitions
    let _ = server.connect().await;
    
    // After connect attempt, status should be Failed (since echo command fails as MCP server)
    match server.status() {
        ServerStatus::Failed(_) => {
            // Expected - connection failed
            assert!(server.retry_count > 0);
            assert!(server.last_retry.is_some());
        }
        status => {
            panic!("Expected Failed status, got: {:?}", status);
        }
    }
}

/// Test McpServer retry logic with various scenarios.
#[tokio::test]
async fn test_mcp_server_retry_logic() {
    let config = ExternalServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("nonexistent-command".to_string()),
        args: Some(vec![]),
        url: None,
    };

    let mut server = McpServer::new(config);
    
    // Test initial retry (should be allowed)
    assert!(server.should_retry());
    
    // Simulate a failure by setting retry count and last retry time
    server.retry_count = 1;
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(5));
    
    // Should retry after sufficient delay (5s is more than 3s delay for retry_count=1)
    assert!(server.should_retry());
    
    // Test retry count limit (3 max retries before waiting 30s)
    server.retry_count = 3;
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(45));
    
    // Should retry after 30s cooldown period
    assert!(server.should_retry());
    
    // Test within cooldown period (should not retry)
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(25));
    assert!(!server.should_retry());
    
    // Test disabled server should never retry
    server.config.enabled = false;
    assert!(!server.should_retry());
}

/// Test McpServer should_retry logic with timing edge cases.
#[tokio::test]
async fn test_mcp_server_retry_timing_edge_cases() {
    let config = ExternalServerConfig {
        name: "test-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("echo".to_string()),
        args: Some(vec![]),
        url: None,
    };

    let mut server = McpServer::new(config);
    
    // Test immediate retry when no last_retry set
    server.retry_count = 0;
    server.last_retry = None;
    assert!(server.should_retry());
    
    // Test retry immediately after failure (within 1s delay for first retry)
    server.retry_count = 0;
    server.last_retry = Some(std::time::Instant::now());
    assert!(!server.should_retry());
    
    // Test after sufficient delay for first retry
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(2));
    assert!(server.should_retry());
    
    // Test second retry timing
    server.retry_count = 1;
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(4));
    assert!(server.should_retry());
    
    // Test third retry timing  
    server.retry_count = 2;
    server.last_retry = Some(std::time::Instant::now() - std::time::Duration::from_secs(6));
    assert!(server.should_retry());
}

/// Test McpServerManager creation with valid configuration.
#[tokio::test]
async fn test_mcp_server_manager_creation() {
    let mut config = McpConfig::default();
    
    // Add a test server configuration
    config.external_servers.push(ExternalServerConfig {
        name: "test-server-1".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("echo".to_string()),
        args: Some(vec!["test".to_string()]),
        url: None,
    });
    
    // Add a disabled server
    config.external_servers.push(ExternalServerConfig {
        name: "disabled-server".to_string(),
        transport: TransportType::Stdio,
        enabled: false,
        command: Some("echo".to_string()),
        args: Some(vec!["disabled".to_string()]),
        url: None,
    });
    
    let tool_registry = Arc::new(RwLock::new(ToolRegistry::new(&config.builtin_tools)));
    let manager = McpServerManager::new(&config, tool_registry).await.unwrap();
    
    // Should have only the enabled server in the manager
    assert_eq!(manager.servers.len(), 1);
    assert!(manager.servers.contains_key("test-server-1"));
    assert!(!manager.servers.contains_key("disabled-server"));
}

/// Test McpServerManager get_server_statuses method.
#[tokio::test]
async fn test_mcp_server_manager_get_statuses() {
    let mut config = McpConfig::default();
    
    config.external_servers.push(ExternalServerConfig {
        name: "status-test-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: Some("echo".to_string()),
        args: Some(vec!["status".to_string()]),
        url: None,
    });
    
    let tool_registry = Arc::new(RwLock::new(ToolRegistry::new(&config.builtin_tools)));
    let manager = McpServerManager::new(&config, tool_registry).await.unwrap();
    
    let statuses = manager.get_server_statuses().await;
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].0, "status-test-server");
    // Initial status should be Disconnected or Failed (since echo isn't an MCP server)
    match &statuses[0].1 {
        ServerStatus::Disconnected => {
            // This could happen if connect wasn't attempted yet
        }
        ServerStatus::Failed(_) => {
            // This is expected since echo command fails as MCP server
        }
        status => {
            panic!("Unexpected status: {:?}", status);
        }
    }
}

/// Test error handling for invalid transport types.
#[tokio::test]
async fn test_invalid_transport_type_error() {
    let config = ExternalServerConfig {
        name: "http-server".to_string(),
        transport: TransportType::Http, // HTTP not supported in rmcp 1.0.0
        enabled: true,
        command: None,
        args: None,
        url: Some("http://localhost:3000".to_string()),
    };

    let mut server = McpServer::new(config);
    let result = server.connect().await;
    
    // Should fail with error about HTTP/SSE not being available
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HTTP/SSE transport not available"));
}

/// Test server with missing command for stdio transport.
#[tokio::test]
async fn test_stdio_transport_missing_command() {
    let config = ExternalServerConfig {
        name: "no-command-server".to_string(),
        transport: TransportType::Stdio,
        enabled: true,
        command: None, // Missing command
        args: Some(vec!["test".to_string()]),
        url: None,
    };

    let mut server = McpServer::new(config);
    let result = server.connect().await;
    
    // Should fail with error about missing command
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires 'command' field"));
}