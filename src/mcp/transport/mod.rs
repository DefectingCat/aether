//! # MCP 传输层
//!
//! 提供与 MCP 服务器通信的传输实现。
//!
//! ## 支持的传输方式
//!
//! - **Stdio**: 通过标准输入输出通信
//! - **HTTP**: 通过 HTTP API 通信
//! - **SSE**: 通过 Server-Sent Events 通信

pub mod http;
pub mod stdio;

pub use stdio::StdioTransport;
