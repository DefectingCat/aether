//! # HTTP/SSE 传输实现
//!
//! 与远程 HTTP/SSE MCP 服务器通信的传输层实现。

use anyhow::{Context, Result};
use rmcp::service::ServiceExt;
use rmcp::service::{Peer, RoleClient};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransport;

use super::super::config::ExternalServerConfig;

/// HTTP/SSE 传输客户端
pub struct HttpTransport {
    peer: Peer<RoleClient>,
    config: ExternalServerConfig,
}

impl HttpTransport {
    /// 创建新的 HTTP 传输客户端
    pub async fn new(config: &ExternalServerConfig) -> Result<Self> {
        let url = config
            .url
            .as_ref()
            .context("HTTP transport requires 'url' field in configuration")?;

        let transport = StreamableHttpClientTransport::from_uri(url.as_str());

        let client = ().serve(transport).await.context("Failed to create MCP client")?;

        let peer = client.peer().clone();

        Ok(Self {
            peer,
            config: config.clone(),
        })
    }

    /// 获取 MCP peer 引用
    pub fn peer(&self) -> &Peer<RoleClient> {
        &self.peer
    }

    /// 获取服务器配置
    pub fn config(&self) -> &ExternalServerConfig {
        &self.config
    }

    /// 关闭连接
    pub async fn close(self) -> Result<()> {
        Ok(())
    }
}
