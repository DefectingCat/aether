//! 命令系统模块
//!
//! 提供命令解析、权限控制、路由分发等功能

mod context;
mod gateway;
mod parser;
mod permission;
mod registry;

pub use context::CommandContext;
pub use gateway::CommandGateway;
pub use parser::{ParsedCommand, Parser};
pub use permission::Permission;
pub use registry::{CommandHandler, CommandRegistry};