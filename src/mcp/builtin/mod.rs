//! # 内置工具模块
//!
//! 提供开箱即用的内置工具实现。

mod web_fetch;

pub use web_fetch::WebFetchTool;

use super::{Tool, ToolSource};

/// 内置工具枚举
pub enum BuiltInTools {
    /// Web Fetch 工具
    WebFetch(WebFetchTool),
}

#[async_trait::async_trait]
impl Tool for BuiltInTools {
    fn definition(&self) -> super::ToolDefinition {
        match self {
            BuiltInTools::WebFetch(tool) => tool.definition(),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> anyhow::Result<super::ToolResult> {
        match self {
            BuiltInTools::WebFetch(tool) => tool.execute(arguments).await,
        }
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
}
