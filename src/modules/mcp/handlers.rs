//! MCP 命令处理器实现

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::command::{CommandHandler, CommandContext, Permission};
use crate::mcp::{McpServerManager, ServerStatus};
use crate::ui::{error, success, warning};

/// MCP 管理命令处理器
pub struct McpHandler {
    mcp_manager: Option<Arc<RwLock<McpServerManager>>>,
}

impl McpHandler {
    /// 创建新的 MCP 命令处理器
    pub fn new(mcp_manager: Option<Arc<RwLock<McpServerManager>>>) -> Self {
        Self { mcp_manager }
    }
}

#[async_trait]
impl CommandHandler for McpHandler {
    fn name(&self) -> &str {
        "mcp"
    }
    
    fn description(&self) -> &str {
        "MCP 服务器管理命令"
    }
    
    fn usage(&self) -> &str {
        "!mcp <子命令>\n\
        子命令:\n\
        - list: 列出所有可用的MCP工具\n\
        - servers: 查看MCP服务器连接状态\n\
        - reload: 重载MCP配置（仅Bot所有者）"
    }
    
    fn permission(&self) -> Permission {
        Permission::Anyone
    }
    
    async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        let subcommand = ctx.args.first().copied().unwrap_or_default();
        
        match subcommand {
            "list" => self.handle_list(ctx).await,
            "servers" => self.handle_servers(ctx).await,
            "reload" => self.handle_reload(ctx).await,
            _ => self.handle_help(ctx).await,
        }
    }
}

impl McpHandler {
    /// 处理 !mcp list 命令
    async fn handle_list(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        if self.mcp_manager.is_none() {
            let html = error("MCP功能未启用");
            return send_html(&ctx.room, &html).await;
        }
        
        let html = success("MCP工具列表功能已启用");
        send_html(&ctx.room, &html).await
    }
    
    /// 处理 !mcp servers 命令
    async fn handle_servers(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        if let Some(manager) = &self.mcp_manager {
            let manager = manager.read().await;
            let statuses = manager.get_server_statuses().await;
            
            if statuses.is_empty() {
                let html = warning("没有配置外部MCP服务器");
                return send_html(&ctx.room, &html).await;
            }
            
            let mut message = "🖥️ MCP服务器状态：\n\n".to_string();
            for (name, status) in statuses {
                let status_icon = match status {
                    ServerStatus::Connected => "✅",
                    ServerStatus::Connecting => "🔄",
                    ServerStatus::Disconnected => "⚪",
                    ServerStatus::Failed(_) => "❌",
                };
                
                let status_text = match status {
                    ServerStatus::Connected => "已连接",
                    ServerStatus::Connecting => "连接中",
                    ServerStatus::Disconnected => "未连接",
                    ServerStatus::Failed(e) => &format!("连接失败: {}", e),
                };
                
                message.push_str(&format!("{} **{}**: {}\n", status_icon, name, status_text));
            }
            
            send_html(&ctx.room, &message).await?;
        } else {
            let html = error("MCP功能未启用");
            send_html(&ctx.room, &html).await?;
        }
        
        Ok(())
    }
    
    /// 处理 !mcp reload 命令
    async fn handle_reload(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        // 检查权限：仅Bot所有者可以重载
        if !Permission::BotOwner
            .check(&ctx.room, &ctx.sender, ctx.bot_owners)
            .await
        {
            let html = error("权限不足，仅Bot所有者可以执行此操作");
            return send_html(&ctx.room, &html).await;
        }
        
        if let Some(manager) = &self.mcp_manager {
            let manager = manager.write().await;
            
            // TODO: 实现配置重载逻辑，需要从配置文件重新加载
            let html = warning("配置重载功能尚未完全实现");
            send_html(&ctx.room, &html).await?;
            
            // 重新连接所有服务器
            info!("Reloading MCP servers by user request: {}", ctx.sender);
            manager.connect_all_servers().await;
            manager.register_all_external_tools().await;
            
            let html = success("已重新连接所有MCP服务器");
            send_html(&ctx.room, &html).await?;
        } else {
            let html = error("MCP功能未启用");
            send_html(&ctx.room, &html).await?;
        }
        
        Ok(())
    }
    
    /// 处理帮助信息
    async fn handle_help(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        let help = format!("📖 MCP管理命令帮助：\n\n{}", self.usage());
        send_html(&ctx.room, &help).await
    }
}

/// 发送 HTML 消息
async fn send_html(room: &matrix_sdk::Room, html: &str) -> anyhow::Result<()> {
    let plain_text = html
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "")
        .chars()
        .take(100)
        .collect::<String>();

    let content = RoomMessageEventContent::text_html(plain_text, html);
    room.send(content).await?;
    Ok(())
}