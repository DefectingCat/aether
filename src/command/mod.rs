//! # 命令系统模块
//!
//! 提供命令解析、权限控制、路由分发等功能。
//!
//! ## 核心类型
//!
//! - [`CommandHandler`][]: 命令处理器 trait，定义命令的执行接口
//! - [`CommandGateway`][]: 命令网关，负责命令路由和分发
//! - [`CommandContext`][]: 命令执行上下文，包含执行所需的所有信息
//! - [`Permission`][]: 权限检查器
//!
//! ## 架构设计
//!
//! ```text
//! 消息 -> Parser -> ParsedCommand
//!                    ↓
//!              CommandGateway
//!                    ↓
//!         Permission Check (Permission)
//!                    ↓
//!              CommandHandler
//!                    ↓
//!              CommandContext
//!                    ↓
//!              执行命令逻辑
//! ```
//!
//! ## 使用示例
//!
//! ```no_run
//! use std::sync::Arc;
//! use aether_matrix::command::{CommandGateway, CommandHandler, CommandContext, Permission};
//! use async_trait::async_trait;
//!
//! // 定义一个简单的命令处理器
//! struct HelpHandler;
//!
//! #[async_trait]
//! impl CommandHandler for HelpHandler {
//!     fn name(&self) -> &str {
//!         "help"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "显示帮助信息"
//!     }
//!
//!     async fn execute(&self, ctx: &CommandContext<'_>) -> anyhow::Result<()> {
//!         // 命令执行逻辑
//!         Ok(())
//!     }
//! }
//!
//! // 创建命令网关并注册处理器
//! let mut gateway = CommandGateway::new("!".to_string(), vec![]);
//! gateway.register(Arc::new(HelpHandler));
//! ```

mod context;
mod gateway;
mod parser;
mod permission;
mod registry;

pub use context::CommandContext;
pub use gateway::CommandGateway;
#[allow(unused_imports)]
pub use parser::ParsedCommand;
#[allow(unused_imports)]
pub use parser::Parser;
pub use permission::Permission;
pub use registry::CommandHandler;
