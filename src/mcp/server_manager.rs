//! # MCP 服务器管理器
//!
//! 管理所有外部 MCP 服务器的连接、生命周期、工具发现和注册。

use anyhow::{Context, Result};
use async_trait::async_trait;
use rmcp::model::{CallToolRequestParams, Tool as McpTool};
use rmcp::service::{Peer, RoleClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::config::{ExternalServerConfig, McpConfig, TransportType};
use super::tool_registry::{Tool, ToolDefinition, ToolRegistry, ToolResult, ToolSource};
use super::transport::StdioTransport;

/// 服务器连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 连接失败
    Failed(String),
}

/// MCP 服务器实例
pub struct McpServer {
    config: ExternalServerConfig,
    peer: Option<Peer<RoleClient>>,
    status: ServerStatus,
    retry_count: u32,
    last_retry: Option<std::time::Instant>,
}

impl McpServer {
    /// 创建新的 MCP 服务器实例
    pub fn new(config: ExternalServerConfig) -> Self {
        Self {
            config,
            peer: None,
            status: ServerStatus::Disconnected,
            retry_count: 0,
            last_retry: None,
        }
    }

    /// 连接到服务器
    pub async fn connect(&mut self) -> Result<()> {
        self.status = ServerStatus::Connecting;
        info!("Connecting to MCP server: {}", self.config.name);

        let result = match self.config.transport {
            TransportType::Stdio => StdioTransport::new(&self.config)
                .await
                .map(|t| t.peer().clone()),
            TransportType::Http | TransportType::Sse => {
                anyhow::bail!("HTTP/SSE transport not available in rmcp 1.0.0")
            }
        };

        match result {
            Ok(peer) => {
                self.peer = Some(peer);
                self.status = ServerStatus::Connected;
                self.retry_count = 0;
                self.last_retry = None;
                info!("Successfully connected to MCP server: {}", self.config.name);
                Ok(())
            }
            Err(e) => {
                let err_msg = e.to_string();
                self.status = ServerStatus::Failed(err_msg.clone());
                self.retry_count += 1;
                self.last_retry = Some(std::time::Instant::now());
                error!(
                    "Failed to connect to MCP server {}: {}",
                    self.config.name, e
                );
                Err(e)
            }
        }
    }

    /// 获取工具列表
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let peer = self.peer.as_ref().context("Not connected to MCP server")?;

        let response = peer
            .list_tools(None)
            .await
            .context("Failed to list tools from MCP server")?;

        Ok(response.tools)
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        let peer = self.peer.as_ref().context("Not connected to MCP server")?;

        let arguments = if arguments.is_object() {
            arguments.as_object().cloned()
        } else {
            None
        };

        let mut params = CallToolRequestParams::new(name.to_string());
        if let Some(args) = arguments {
            params = params.with_arguments(args);
        }

        let response = peer.call_tool(params).await.context("Tool call failed")?;

        if response.is_error.unwrap_or(false) {
            Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some(
                    response
                        .content
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
            })
        } else {
            Ok(ToolResult {
                success: true,
                content: response
                    .content
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join("\n"),
                error: None,
            })
        }
    }

    /// 获取服务器状态
    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    /// 获取服务器配置
    pub fn config(&self) -> &ExternalServerConfig {
        &self.config
    }

    /// 是否应该重试连接
    pub fn should_retry(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        if self.retry_count >= 3 {
            // 最多重试3次，之后30秒再尝试
            if let Some(last_retry) = self.last_retry {
                return last_retry.elapsed() > std::time::Duration::from_secs(30);
            }
            return true;
        }

        // 重试间隔：1s, 3s, 5s
        let delay = match self.retry_count {
            0 => 1,
            1 => 3,
            _ => 5,
        };

        if let Some(last_retry) = self.last_retry {
            last_retry.elapsed() > std::time::Duration::from_secs(delay)
        } else {
            true
        }
    }
}

/// 外部 MCP 工具包装器
struct ExternalMcpTool {
    server_name: String,
    tool: McpTool,
    server: Arc<RwLock<McpServer>>,
}

impl ExternalMcpTool {
    pub fn new(server_name: String, tool: McpTool, server: Arc<RwLock<McpServer>>) -> Self {
        Self {
            server_name,
            tool,
            server,
        }
    }
}

#[async_trait]
impl Tool for ExternalMcpTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: format!("{}__{}", self.server_name, self.tool.name),
            description: self
                .tool
                .description
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_default(),
            parameters: serde_json::to_value(self.tool.input_schema.as_ref().clone())
                .unwrap_or_default(),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult> {
        let server = self.server.read().await;

        if server.status() != &ServerStatus::Connected {
            return Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some(format!("MCP server {} is not connected", self.server_name)),
            });
        }

        // 调用工具时使用原始工具名称，不带前缀
        server.call_tool(&self.tool.name, arguments).await
    }

    fn source(&self) -> ToolSource {
        ToolSource::ExternalMcp(self.server_name.clone())
    }
}

/// MCP 服务器管理器
pub struct McpServerManager {
    servers: HashMap<String, Arc<RwLock<McpServer>>>,
    tool_registry: Arc<RwLock<ToolRegistry>>,
}

impl McpServerManager {
    /// 创建新的 MCP 服务器管理器
    pub async fn new(config: &McpConfig, tool_registry: Arc<RwLock<ToolRegistry>>) -> Result<Self> {
        let mut servers = HashMap::new();

        for server_config in &config.external_servers {
            if !server_config.enabled {
                info!("MCP server {} is disabled, skipping", server_config.name);
                continue;
            }

            let server = McpServer::new(server_config.clone());
            servers.insert(server_config.name.clone(), Arc::new(RwLock::new(server)));
        }

        let manager = Self {
            servers,
            tool_registry,
        };

        // 启动时连接所有服务器并注册工具
        manager.connect_all_servers().await;
        manager.register_all_external_tools().await;

        Ok(manager)
    }

    /// 连接所有配置的服务器
    pub async fn connect_all_servers(&self) {
        let mut handles = Vec::new();

        for (name, server) in &self.servers {
            let name = name.clone();
            let server = server.clone();

            handles.push(tokio::spawn(async move {
                let mut server = server.write().await;
                if server.should_retry()
                    && let Err(e) = server.connect().await
                {
                    warn!("Failed to connect to MCP server {}: {}", name, e);
                }
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    /// 注册所有外部工具到工具注册表
    pub async fn register_all_external_tools(&self) {
        let mut registry = self.tool_registry.write().await;

        for (server_name, server) in &self.servers {
            let server_guard = server.read().await;

            if server_guard.status() != &ServerStatus::Connected {
                warn!(
                    "MCP server {} is not connected, skipping tool registration",
                    server_name
                );
                continue;
            }

            match server_guard.list_tools().await {
                Ok(tools) => {
                    info!(
                        "Found {} tools from MCP server {}",
                        tools.len(),
                        server_name
                    );

                    for tool in tools {
                        let external_tool =
                            ExternalMcpTool::new(server_name.clone(), tool, server.clone());
                        registry.register(Arc::new(external_tool));
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to list tools from MCP server {}: {}",
                        server_name, e
                    );
                }
            }
        }
    }

    /// 获取所有服务器状态
    pub async fn get_server_statuses(&self) -> Vec<(String, ServerStatus)> {
        let mut statuses = Vec::new();

        for (name, server) in &self.servers {
            let server_guard = server.read().await;
            statuses.push((name.clone(), server_guard.status().clone()));
        }

        statuses
    }

    /// 重载配置
    pub async fn reload_config(&mut self, new_config: &McpConfig) -> Result<()> {
        info!("Reloading MCP server configuration");

        // 移除不再配置的服务器
        let existing_names: Vec<_> = self.servers.keys().cloned().collect();
        for name in existing_names {
            if !new_config.external_servers.iter().any(|s| s.name == name) {
                info!("Removing MCP server: {}", name);
                self.servers.remove(&name);
            }
        }

        // 添加或更新服务器
        for server_config in &new_config.external_servers {
            if !server_config.enabled {
                continue;
            }

            if let Some(existing) = self.servers.get(&server_config.name) {
                // 更新配置
                let mut existing_guard = existing.write().await;
                existing_guard.config = server_config.clone();
                // 如果配置变化了，重新连接
                if existing_guard.status() == &ServerStatus::Connected {
                    existing_guard.status = ServerStatus::Disconnected;
                }
            } else {
                // 新增服务器
                let server = McpServer::new(server_config.clone());
                self.servers
                    .insert(server_config.name.clone(), Arc::new(RwLock::new(server)));
            }
        }

        // 重新连接所有服务器
        self.connect_all_servers().await;

        // 清空注册表中的外部工具，重新注册
        {
            let mut registry = self.tool_registry.write().await;
            registry.remove_all_external_tools();
        }
        self.register_all_external_tools().await;

        Ok(())
    }
}
