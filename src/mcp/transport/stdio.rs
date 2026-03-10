//! # Stdio 传输实现
//!
//! 与子进程模式的 MCP 服务器通信的传输层实现。

use anyhow::{Context, Result};
use rmcp::{
    service::{Peer, RoleClient, ServiceExt},
    transport::TokioChildProcess,
};
use std::process::Stdio;
use tokio::process::Command;

use super::super::config::ExternalServerConfig;

/// Stdio 传输客户端
pub struct StdioTransport {
    peer: Peer<RoleClient>,
    config: ExternalServerConfig,
}

impl StdioTransport {
    /// 创建新的 Stdio 传输客户端
    pub async fn new(config: &ExternalServerConfig) -> Result<Self> {
        let command = config
            .command
            .as_ref()
            .context("Stdio transport requires 'command' field in configuration")?;

        let empty_args = vec![];
        let args = config.args.as_ref().unwrap_or(&empty_args);

        // 构建子进程命令
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        // 创建子进程传输
        let transport = TokioChildProcess::new(cmd)
            .context("Failed to create child process transport for MCP server")?;

        // 创建 MCP 客户端服务
        // 使用 () 作为空的 ClientHandler
        let service = ().serve(transport).await.context("Failed to create MCP client")?;

        Ok(Self {
            peer: service.peer().clone(),
            config: config.clone(),
        })
    }

    /// 获取 MCP 客户端 Peer 引用
    pub fn peer(&self) -> &Peer<RoleClient> {
        &self.peer
    }

    /// 获取服务器配置
    pub fn config(&self) -> &ExternalServerConfig {
        &self.config
    }
}
