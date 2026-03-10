use aether_matrix::mcp::config::{ExternalServerConfig, TransportType};
use aether_matrix::mcp::{BuiltinToolsConfig, McpConfig, WebFetchConfig};
use std::env;

#[test]
fn test_mcp_config_defaults() {
    let config = McpConfig::default();
    assert!(config.enabled);
    assert!(config.builtin_tools.enabled);
    assert_eq!(config.external_servers.len(), 0);
}

#[test]
fn test_builtin_tools_config_defaults() {
    let config = BuiltinToolsConfig::default();
    assert!(config.enabled);
}

#[test]
fn test_web_fetch_config_defaults() {
    let config = WebFetchConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_length, 10000);
    assert_eq!(config.timeout, 10);
}

#[test]
fn test_env_overrides_mcp_enabled_false() {
    env::set_var("MCP_ENABLED", "false");
    let mut config = McpConfig::default();
    config.apply_env_overrides();
    assert!(!config.enabled);
    env::remove_var("MCP_ENABLED");
}

#[test]
fn test_env_overrides_mcp_enabled_true() {
    env::set_var("MCP_ENABLED", "true");
    let mut config = McpConfig::default();
    config.apply_env_overrides();
    assert!(config.enabled);
    env::remove_var("MCP_ENABLED");
}

#[test]
fn test_env_overrides_mcp_enabled_case_insensitive() {
    env::set_var("MCP_ENABLED", "FALSE");
    let mut config = McpConfig::default();
    config.apply_env_overrides();
    assert!(!config.enabled);
    env::remove_var("MCP_ENABLED");
}

#[test]
fn test_env_overrides_builtin_tools_disabled() {
    env::set_var("MCP_BUILTIN_TOOLS_ENABLED", "false");
    let mut config = BuiltinToolsConfig::default();
    config.apply_env_overrides();
    assert!(!config.enabled);
    env::remove_var("MCP_BUILTIN_TOOLS_ENABLED");
}

#[test]
fn test_env_overrides_web_fetch_all_values() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_ENABLED", "false");
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "5000");
    env::set_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT", "30");

    let mut config = WebFetchConfig::default();
    config.apply_env_overrides();

    assert!(!config.enabled);
    assert_eq!(config.max_length, 5000);
    assert_eq!(config.timeout, 30);

    env::remove_var("MCP_BUILTIN_WEB_FETCH_ENABLED");
    env::remove_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH");
    env::remove_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT");
}

#[test]
fn test_env_overrides_invalid_numeric_values_fallback_to_defaults() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "invalid");
    env::set_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT", "not_a_number");

    let mut config = WebFetchConfig::default();
    config.apply_env_overrides();

    assert_eq!(config.max_length, 10000);
    assert_eq!(config.timeout, 10);

    env::remove_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH");
    env::remove_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT");
}

#[test]
fn test_toml_config_parsing_full_config() {
    let toml_str = r#"
        enabled = false

        [builtin_tools]
        enabled = false

        [builtin_tools.web_fetch]
        enabled = false
        max_length = 20000
        timeout = 60

        [[external_servers]]
        name = "test-server"
        transport = "stdio"
        command = "test-command"
        args = ["arg1", "arg2"]
        enabled = true
    "#;

    let config: McpConfig = toml::from_str(toml_str).expect("TOML parsing should succeed");

    assert!(!config.enabled);
    assert!(!config.builtin_tools.enabled);
    assert!(!config.builtin_tools.web_fetch.enabled);
    assert_eq!(config.builtin_tools.web_fetch.max_length, 20000);
    assert_eq!(config.builtin_tools.web_fetch.timeout, 60);
    assert_eq!(config.external_servers.len(), 1);

    let server = &config.external_servers[0];
    assert_eq!(server.name, "test-server");
    assert_eq!(server.transport, TransportType::Stdio);
    assert_eq!(server.command, Some("test-command".to_string()));
    assert_eq!(
        server.args,
        Some(vec!["arg1".to_string(), "arg2".to_string()])
    );
    assert!(server.enabled);
}

#[test]
fn test_transport_type_lowercase_parsing() {
    let stdio: TransportType = serde_json::from_str("\"stdio\"").unwrap();
    assert_eq!(stdio, TransportType::Stdio);

    let http: TransportType = serde_json::from_str("\"http\"").unwrap();
    assert_eq!(http, TransportType::Http);

    let sse: TransportType = serde_json::from_str("\"sse\"").unwrap();
    assert_eq!(sse, TransportType::Sse);
}

#[test]
fn test_transport_type_uppercase_fails() {
    let result = serde_json::from_str::<TransportType>("\"Stdio\"");
    assert!(result.is_err());
}

#[test]
fn test_backward_compatible_json_servers_parsing() {
    let json_servers = r#"[
        {
            "name": "filesystem",
            "transport": "stdio",
            "command": "mcp-fs-server",
            "args": ["/home/user"],
            "enabled": true
        }
    ]"#;

    env::set_var("MCP_EXTERNAL_SERVERS", json_servers);

    let mut config = McpConfig::default();
    config.apply_env_overrides();

    assert_eq!(config.external_servers.len(), 1);
    let server = &config.external_servers[0];
    assert_eq!(server.name, "filesystem");
    assert_eq!(server.transport, TransportType::Stdio);
    assert_eq!(server.command, Some("mcp-fs-server".to_string()));
    assert_eq!(server.args, Some(vec!["/home/user".to_string()]));
    assert!(server.enabled);

    env::remove_var("MCP_EXTERNAL_SERVERS");
}

#[test]
fn test_invalid_json_servers_graceful_handling() {
    env::set_var("MCP_EXTERNAL_SERVERS", "invalid json {");

    let mut config = McpConfig::default();
    config.apply_env_overrides();

    assert_eq!(config.external_servers.len(), 0);

    env::remove_var("MCP_EXTERNAL_SERVERS");
}

#[test]
fn test_web_fetch_zero_values() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "0");
    env::set_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT", "0");

    let mut config = WebFetchConfig::default();
    config.apply_env_overrides();

    assert_eq!(config.max_length, 0);
    assert_eq!(config.timeout, 0);

    env::remove_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH");
    env::remove_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT");
}

#[test]
fn test_web_fetch_large_values() {
    env::set_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH", "1000000");
    env::set_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT", "3600");

    let mut config = WebFetchConfig::default();
    config.apply_env_overrides();

    assert_eq!(config.max_length, 1000000);
    assert_eq!(config.timeout, 3600);

    env::remove_var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH");
    env::remove_var("MCP_BUILTIN_WEB_FETCH_TIMEOUT");
}
