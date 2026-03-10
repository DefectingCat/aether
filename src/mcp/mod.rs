//! # MCP (Model Context Protocol) 集成模块
//!
//! 提供 MCP Client 功能，支持内置工具和外部 MCP 服务器集成，扩展 AI 能力。
//!
//! ## 功能特性
//!
//! - **内置工具**: 开箱即用的工具（如 [`web_fetch`](crate::mcp::builtin::WebFetchTool)）
//! - **外部 MCP**: 支持连接外部 MCP 服务器（stdio/HTTP/SSE 传输）
//! - **统一接口**: 内置和外部工具使用相同的 [`Tool`] 接口
//! - **优雅降级**: MCP 不可用时自动降级到普通聊天模式
//! - **自动重试**: 连接失败时自动重试，带指数退避
//!
//! ## 模块结构
//!
//! - [`config`]: MCP 配置管理，支持 TOML 和环境变量
//! - [`tool_registry`]: 工具注册表，统一管理所有工具
//! - [`builtin`]: 内置工具实现（web_fetch 等）
//! - [`transport`]: MCP 传输层（Stdio/HTTP/SSE）
//! - [`server_manager`]: MCP 服务器管理器，处理连接和工具发现
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────┐
//! │   AiService     │
//! └────────┬────────┘
//!          │
//!          ├──► ToolRegistry (统一工具接口)
//!          │       ├── BuiltInTools (web_fetch, ...)
//!          │       └── External MCP Tools (来自服务器)
//!          │
//!          └──► McpServerManager
//!                  ├── Server 1 (stdio)
//!                  ├── Server 2 (http)
//!                  └── Server 3 (sse)
//! ```
//!
//! ## 使用示例
//!
//! ### 配置示例 (config.toml)
//!
//! ```toml
//! [mcp]
//! enabled = true
//!
//! [mcp.builtin_tools]
//! enabled = true
//!
//! [mcp.builtin_tools.web_fetch]
//! enabled = true
//! max_length = 10000
//! timeout = 10
//!
//! [[mcp.external_servers]]
//! name = "filesystem"
//! transport = "stdio"
//! command = "npx"
//! args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"]
//!
//! [[mcp.external_servers]]
//! name = "database"
//! transport = "http"
//! url = "http://localhost:3000/mcp"
//! ```
//!
//! ## 工具调用流程
//!
//! 1. AI 决定调用工具（通过 function call）
//! 2. [`ToolRegistry`] 查找工具
//! 3. 根据工具来源调用：
//!    - 内置工具：直接执行 Rust 代码
//!    - 外部工具：通过 MCP 协议调用远程服务器
//! 4. 返回结果给 AI，AI 生成最终响应
//!
//! ## 参考资源
//!
//! - [MCP 官方文档](https://modelcontextprotocol.io/)
//! - [rmcp SDK](https://crates.io/crates/rmcp)

#[allow(dead_code)]
pub mod builtin;
pub mod config;
#[allow(dead_code)]
pub mod server_manager;
#[allow(dead_code)]
pub mod tool_registry;
#[allow(dead_code)]
pub mod transport;

pub use config::{BuiltinToolsConfig, McpConfig, WebFetchConfig};
#[allow(unused_imports)]
pub use config::{ExternalServerConfig, TransportType};
#[allow(unused_imports)]
pub use server_manager::{McpServer, McpServerManager, ServerStatus};
#[allow(unused_imports)]
pub use tool_registry::{Tool, ToolDefinition, ToolRegistry, ToolResult, ToolSource};
