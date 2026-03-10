//! MCP 命令处理器实现

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

use crate::command::{CommandHandler, CommandContext, Permission};
use crate::mcp::{McpServerManager, ServerStatus};
use crate::ui::{error, success, warning};

/// MCP 管理命令处理器。
///
/// 提供与 Model Context Protocol (MCP) 相关的管理命令，包括：
/// - 查看可用工具列表
/// - 查看服务器连接状态
/// - 重载 MCP 配置
///
/// # 子命令
///
/// | 命令 | 说明 | 权限 |
/// |------|------|------|
/// | `!mcp list` | 列出所有可用的 MCP 工具 | Anyone |
/// | `!mcp servers` | 查看 MCP 服务器连接状态 | Anyone |
/// | `!mcp reload` | 重载 MCP 配置并重连服务器 | BotOwner |
///
/// # 权限
///
/// 基础命令（list、servers）任何房间成员都可以执行。
/// `reload` 子命令需要 Bot 所有者权限，在运行时检查。
///
/// # Example
///
/// ```ignore
/// use aether_matrix::command::{CommandHandler, CommandContext, Permission};
/// use aether_matrix::mcp::McpServerManager;
/// use async_trait::async_trait;
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
///
/// /// MCP 管理命令处理器。
/// pub struct McpHandler<T: AiServiceTrait> {
///     mcp_manager: Option<Arc<RwLock<McpServerManager>>>,
///     ai_service: Option<T>,
/// }
///
/// #[async_trait]
/// impl<T: AiServiceTrait + 'static> CommandHandler for McpHandler<T> {
///     fn name(&self) -> &str {
///         "mcp"
///     }
///
///     fn description(&self) -> &str {
///         "MCP 服务器管理命令"
///     }
///
///     fn usage(&self) -> &str {
///         "!mcp <子命令>\n\
///         子命令:\n\
///         - list: 列出所有可用的MCP工具\n\
///         - servers: 查看MCP服务器连接状态\n\
///         - reload: 重载MCP配置（仅Bot所有者）"
///     }
///
///     fn permission(&self) -> Permission {
///         Permission::Anyone
///     }
///
///     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
///         // 根据子命令分发处理
///         Ok(())
///     }
/// }
/// ```
pub struct McpHandler<T: crate::traits::AiServiceTrait> {
    mcp_manager: Option<Arc<RwLock<McpServerManager>>>,
    ai_service: Option<T>,
}

impl<T: crate::traits::AiServiceTrait> McpHandler<T> {
    /// 创建新的 MCP 命令处理器。
    ///
    /// # Arguments
    ///
    /// * `mcp_manager` - MCP 服务器管理器，用于管理外部 MCP 服务器连接
    /// * `ai_service` - AI 服务实例，用于获取可用工具列表
    pub fn new(
        mcp_manager: Option<Arc<RwLock<McpServerManager>>>,
        ai_service: Option<T>,
    ) -> Self {
        Self { mcp_manager, ai_service }
    }
}

#[async_trait]
impl<T: crate::traits::AiServiceTrait + 'static> CommandHandler for McpHandler<T> {
    /// 命令名称（不含前缀）。
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aether_matrix::modules::mcp::McpHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// // 需要 McpServerManager 和 AiService 实例
    /// let handler: McpHandler<_> = McpHandler::new(None, None);
    /// assert_eq!(handler.name(), "mcp");
    /// ```
    fn name(&self) -> &str {
        "mcp"
    }
    
    /// 命令描述。
    ///
    /// 用于帮助信息，简要说明命令功能。
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aether_matrix::modules::mcp::McpHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler: McpHandler<_> = McpHandler::new(None, None);
    /// assert!(!handler.description().is_empty());
    /// ```
    fn description(&self) -> &str {
        "MCP 服务器管理命令"
    }
    
    /// 使用说明。
    ///
    /// 用于帮助信息，说明命令的参数和子命令。
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aether_matrix::modules::mcp::McpHandler;
    /// use aether_matrix::command::CommandHandler;
    ///
    /// let handler: McpHandler<_> = McpHandler::new(None, None);
    /// assert!(handler.usage().contains("list"));
    /// ```
    fn usage(&self) -> &str {
        "!mcp <子命令>\n\
        子命令:\n\
        - list: 列出所有可用的MCP工具\n\
        - servers: 查看MCP服务器连接状态\n\
        - reload: 重载MCP配置（仅Bot所有者）"
    }
    
    /// 所需权限级别。
    ///
    /// 返回 `Anyone`，基础子命令任何房间成员都可执行。
    /// `reload` 子命令的权限在运行时单独检查。
    ///
    /// # Example
    ///
    /// ```ignore
    /// use aether_matrix::modules::mcp::McpHandler;
    /// use aether_matrix::command::{CommandHandler, Permission};
    ///
    /// let handler: McpHandler<_> = McpHandler::new(None, None);
    /// assert_eq!(handler.permission(), Permission::Anyone);
    /// ```
    fn permission(&self) -> Permission {
        Permission::Anyone
    }
    
    /// 执行命令。
    ///
    /// 根据子命令分发到对应的处理方法：
    /// - `list` → 列出可用的 MCP 工具
    /// - `servers` → 显示服务器连接状态
    /// - `reload` → 重载配置并重连（需要 BotOwner 权限）
    /// - 其他 → 显示帮助信息
    ///
    /// # Arguments
    ///
    /// * `ctx` - 命令执行上下文，包含房间、发送者、参数等信息
    ///
    /// # Returns
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
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

impl<T: crate::traits::AiServiceTrait> McpHandler<T> {
    /// 处理 !mcp list 命令
    async fn handle_list(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
        let ai_service = match &self.ai_service {
            Some(svc) => svc,
            None => {
                let html = error("MCP功能未启用");
                return send_html(&ctx.room, &html).await;
            }
        };
        
        let tools = ai_service.list_mcp_tools().await;
        
        if tools.is_empty() {
            let html = warning("暂无可用工具");
            return send_html(&ctx.room, &html).await;
        }
        
        let tool_count = tools.len();
        let mut message = "🔧 **可用工具列表**：\n\n".to_string();
        for tool in tools {
            let desc = tool.description.lines().next().unwrap_or("无描述");
            message.push_str(&format!("• **{}**: {}\n", tool.name, desc));
        }
        
        message.push_str(&format!("\n📊 共 {} 个工具可用", tool_count));
        send_html(&ctx.room, &message).await
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