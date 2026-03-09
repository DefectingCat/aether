//! # MCP 配置
//!
//! 管理 MCP 功能的所有配置项，包括内置工具和外部 MCP 服务器。

use serde::{Deserialize, Serialize};

/// MCP 总配置
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// 是否启用 MCP 功能
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,

    /// 内置工具配置
    #[serde(default)]
    pub builtin_tools: BuiltinToolsConfig,

    /// 外部 MCP 服务器配置
    #[serde(default)]
    pub external_servers: Vec<ExternalServerConfig>,
}

fn default_mcp_enabled() -> bool {
    true
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            builtin_tools: BuiltinToolsConfig::default(),
            external_servers: Vec::new(),
        }
    }
}

/// 内置工具配置
#[derive(Debug, Clone, Deserialize)]
pub struct BuiltinToolsConfig {
    /// 是否启用内置工具
    #[serde(default = "default_builtin_enabled")]
    pub enabled: bool,

    /// web_fetch 工具配置
    #[serde(default)]
    pub web_fetch: WebFetchConfig,
}

fn default_builtin_enabled() -> bool {
    true
}

impl Default for BuiltinToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            web_fetch: WebFetchConfig::default(),
        }
    }
}

/// Web Fetch 工具配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebFetchConfig {
    /// 是否启用 web_fetch 工具
    #[serde(default = "default_web_fetch_enabled")]
    pub enabled: bool,

    /// 最大内容长度（字符数）
    #[serde(default = "default_max_length")]
    pub max_length: usize,

    /// HTTP 请求超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_web_fetch_enabled() -> bool {
    true
}

fn default_max_length() -> usize {
    10000
}

fn default_timeout() -> u64 {
    10
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_length: 10000,
            timeout: 10,
        }
    }
}

/// 外部 MCP 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalServerConfig {
    /// 服务器名称
    pub name: String,

    /// 传输类型
    pub transport: TransportType,

    /// 是否启用
    #[serde(default = "default_server_enabled")]
    pub enabled: bool,

    /// Stdio 传输：命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Stdio 传输：参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// HTTP/SSE 传输：URL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub url: Option<String>,
}

fn default_server_enabled() -> bool {
    true
}

/// 传输类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// Stdio 传输（通过标准输入输出）
    Stdio,
    /// HTTP 传输
    Http,
    /// SSE (Server-Sent Events) 传输
    Sse,
}

impl McpConfig {
    /// 环境变量覆盖（由主 Config 调用）
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }

        self.builtin_tools.apply_env_overrides();

        // 保留 MCP_EXTERNAL_SERVERS JSON 支持（向后兼容，标记为 deprecated）
        if let Ok(json) = std::env::var("MCP_EXTERNAL_SERVERS") {
            match serde_json::from_str(&json) {
                Ok(servers) => {
                    tracing::warn!(
                        "MCP_EXTERNAL_SERVERS 环境变量已弃用，请使用 config.toml 的 [[mcp.external_servers]] 配置外部服务器"
                    );
                    self.external_servers = servers;
                }
                Err(e) => {
                    tracing::warn!("MCP_EXTERNAL_SERVERS JSON 解析失败: {}", e);
                }
            }
        }
    }
}

impl BuiltinToolsConfig {
    /// 环境变量覆盖（由 McpConfig 调用）
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_BUILTIN_TOOLS_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }
        self.web_fetch.apply_env_overrides();
    }
}

impl WebFetchConfig {
    /// 环境变量覆盖（由 BuiltinToolsConfig 调用）
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_ENABLED") {
            self.enabled = v.to_lowercase() != "false";
        }
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_MAX_LENGTH")
            && let Ok(n) = v.parse()
        {
            self.max_length = n;
        }
        if let Ok(v) = std::env::var("MCP_BUILTIN_WEB_FETCH_TIMEOUT")
            && let Ok(n) = v.parse()
        {
            self.timeout = n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpConfig::default();
        assert!(config.enabled);
        assert!(config.builtin_tools.enabled);
        assert!(config.builtin_tools.web_fetch.enabled);
    }

    #[test]
    fn test_web_fetch_config_defaults() {
        let config = WebFetchConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_length, 10000);
        assert_eq!(config.timeout, 10);
    }

    #[test]
    fn test_transport_type_deserialization() {
        let json = r#""stdio""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Stdio);

        let json = r#""http""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Http);

        let json = r#""sse""#;
        let transport: TransportType = serde_json::from_str(json).unwrap();
        assert_eq!(transport, TransportType::Sse);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
            enabled = true

            [builtin_tools]
            enabled = true

            [builtin_tools.web_fetch]
            enabled = true
            max_length = 20000
            timeout = 30

            [[external_servers]]
            name = "filesystem"
            transport = "stdio"
            command = "mcp-server-filesystem"
            args = ["/home/user/documents"]
            enabled = true

            [[external_servers]]
            name = "database"
            transport = "http"
            url = "http://localhost:3000/mcp"
            enabled = true
        "#;

        let config: McpConfig = toml::from_str(toml_str).expect("TOML 解析应成功");
        assert!(config.enabled);
        assert!(config.builtin_tools.enabled);
        assert!(config.builtin_tools.web_fetch.enabled);
        assert_eq!(config.builtin_tools.web_fetch.max_length, 20000);
        assert_eq!(config.builtin_tools.web_fetch.timeout, 30);
        assert_eq!(config.external_servers.len(), 2);

        assert_eq!(config.external_servers[0].name, "filesystem");
        assert_eq!(config.external_servers[0].transport, TransportType::Stdio);
        assert_eq!(
            config.external_servers[0].command,
            Some("mcp-server-filesystem".to_string())
        );
        assert_eq!(
            config.external_servers[0].args,
            Some(vec!["/home/user/documents".to_string()])
        );

        assert_eq!(config.external_servers[1].name, "database");
        assert_eq!(config.external_servers[1].transport, TransportType::Http);
        assert_eq!(
            config.external_servers[1].url,
            Some("http://localhost:3000/mcp".to_string())
        );
    }
}
