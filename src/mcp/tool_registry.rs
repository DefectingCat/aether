//! # 工具注册表
//!
//! 统一管理所有工具（内置和外部），提供工具注册、查询和执行功能。

use anyhow::Result;
use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObject};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::builtin::BuiltInTools;

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 执行是否成功
    pub success: bool,
    /// 工具返回的内容
    pub content: String,
    /// 错误信息（如果有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 工具定义（OpenAI 兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数的 JSON Schema
    pub parameters: serde_json::Value,
}

/// 工具来源
#[derive(Debug, Clone, PartialEq)]
pub enum ToolSource {
    /// 内置工具
    BuiltIn,
    /// 外部 MCP 服务器（包含服务器名称）
    ExternalMcp(String),
}

/// 统一的工具接口
///
/// 所有工具（内置和外部）都必须实现此 trait。
#[async_trait]
pub trait Tool: Send + Sync {
    /// 获取工具定义
    fn definition(&self) -> ToolDefinition;

    /// 执行工具
    ///
    /// # Arguments
    ///
    /// * `arguments` - 工具参数（JSON 格式）
    ///
    /// # Returns
    ///
    /// 返回工具执行结果
    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult>;

    /// 获取工具来源
    fn source(&self) -> ToolSource;
}

/// 工具注册表
///
/// 统一管理所有工具，支持内置工具和外部 MCP 工具的注册和调用。
pub struct ToolRegistry {
    /// 工具映射表（工具名称 -> 工具实例）
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    ///
    /// 会自动注册所有启用的内置工具。
    pub fn new(builtin_config: &super::BuiltinToolsConfig) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // 注册内置工具
        if builtin_config.enabled {
            registry.register_builtin_tools(builtin_config);
        }

        registry
    }

    /// 注册内置工具
    fn register_builtin_tools(&mut self, config: &super::BuiltinToolsConfig) {
        // 注册 web_fetch 工具
        if config.web_fetch.enabled {
            self.register(Arc::new(BuiltInTools::WebFetch(
                super::builtin::WebFetchTool::new(config.web_fetch.clone()),
            )));
        }

        // 未来可以添加更多内置工具
    }

    /// 注册工具
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }

    /// 注册外部 MCP 工具
    pub fn register_mcp_tool(&mut self, tool: Arc<dyn Tool>) {
        self.register(tool);
    }

    /// 获取所有工具定义（OpenAI 格式）
    pub fn to_openai_tools(&self) -> Vec<ChatCompletionTools> {
        self.tools
            .values()
            .map(|tool| {
                let def = tool.definition();
                ChatCompletionTools::Function(ChatCompletionTool {
                    function: FunctionObject {
                        name: def.name,
                        description: Some(def.description),
                        parameters: Some(def.parameters),
                        strict: None,
                    },
                })
            })
            .collect()
    }

    /// 执行工具
    ///
    /// # Arguments
    ///
    /// * `tool_name` - 工具名称
    /// * `arguments` - 工具参数
    ///
    /// # Returns
    ///
    /// 返回工具执行结果
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolResult> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        tool.execute(arguments).await
    }

    /// 获取工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// 检查工具是否存在
    pub fn contains(&self, tool_name: &str) -> bool {
        self.tools.contains_key(tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let config = super::super::BuiltinToolsConfig::default();
        let registry = ToolRegistry::new(&config);

        // 默认配置下，内置工具应该是启用的
        assert!(registry.len() > 0);
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult {
            success: true,
            content: "test content".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test content"));
    }
}
