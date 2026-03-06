# MCP (Model Context Protocol) 集成方案

> **版本**: 1.0  
> **日期**: 2025-03-06  
> **目标**: 为 Aether Matrix Bot 添加 MCP Client 功能，支持内置工具和外部 MCP 服务器集成

---

## 一、概述

### 1.1 设计目标

- **MCP Client**: 让 AI 能够调用外部工具（MCP 服务器）和内置工具
- **稳定性优先**: 现有功能不受影响，MCP 功能完全可选
- **优雅降级**: MCP 不可用时自动降级，不影响用户体验
- **统一接口**: 内置工具和外部工具使用相同的调用接口

### 1.2 核心特性

✅ 支持内置工具（开箱即用）  
✅ 支持外部 MCP 服务器（Stdio/HTTP/SSE）  
✅ 统一的工具注册和管理  
✅ OpenAI Function Calling 集成  
✅ 完全可选，可配置

---

## 二、架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────┐
│                Aether Matrix Bot                     │
│                                                      │
│  ┌────────────────────────────────────────────────┐ │
│  │           EventHandler (不变)                   │ │
│  └────────────────────────────────────────────────┘ │
│                       ↓                              │
│  ┌────────────────────────────────────────────────┐ │
│  │           AiService (扩展)                      │ │
│  │  ┌──────────────────────────────────────────┐  │ │
│  │  │  ConversationManager (支持 tool 消息)    │  │ │
│  │  └──────────────────────────────────────────┘  │ │
│  │  ┌──────────────────────────────────────────┐  │ │
│  │  │  McpClientManager (新增)                 │  │ │
│  │  │  ┌────────────────────────────────────┐  │  │ │
│  │  │  │  Tool Registry (统一管理)          │  │  │ │
│  │  │  │  - 内置工具 (Built-in)             │  │  │ │
│  │  │  │  - 外部 MCP 工具 (External)        │  │  │ │
│  │  │  └────────────────────────────────────┘  │  │ │
│  │  │  ┌────────────────────────────────────┐  │  │ │
│  │  │  │  Tool Executor                     │  │  │ │
│  │  │  │  - 路由到对应的工具提供者          │  │  │ │
│  │  │  └────────────────────────────────────┘  │  │ │
│  │  └──────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────┘ │
│                       ↓                              │
│  ┌────────────────────────────────────────────────┐ │
│  │    OpenAI API (with Function Calling)          │ │
│  └────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
        ↓                    ↓                   ↓
  ┌──────────┐        ┌──────────┐        ┌──────────┐
  │ 内置工具 │        │ MCP Stdio│        │ MCP HTTP │
  │ - fetch  │        │  Server  │        │  Server  │
  │ - 更多...│        └──────────┘        └──────────┘
  └──────────┘
```

### 2.2 Tool Calling 工作流

```
用户消息 → AI 判断 → 需要工具？
                           ↓ 是
                    选择工具（内置/外部）
                           ↓
                    调用工具执行
                           ↓
                    返回结果 → AI 继续处理
                           ↓
                    最终回复给用户
```

### 2.3 降级策略

| 场景            | 降级行为                   |
| --------------- | -------------------------- |
| MCP 完全禁用    | 纯文本对话，无工具调用     |
| 外部 MCP 不可用 | 仅使用内置工具             |
| 内置工具失败    | 返回错误给 AI，AI 可换策略 |
| Tool 执行超时   | 返回超时错误，继续对话     |

---

## 三、模块设计

### 3.1 目录结构

```
src/
├── mcp/                          # MCP 模块
│   ├── mod.rs                    # 模块入口
│   ├── client.rs                 # MCP Client 实现
│   ├── transport/                # 传输层
│   │   ├── mod.rs
│   │   ├── stdio.rs              # Stdio 传输
│   │   ├── sse.rs                # SSE 传输
│   │   └── http.rs               # HTTP 传输
│   ├── tool_registry.rs          # Tool 注册表（统一管理）
│   ├── tool_executor.rs          # Tool 执行器
│   ├── config.rs                 # MCP 配置
│   └── builtin/                  # 内置工具
│       ├── mod.rs                # 内置工具模块入口
│       ├── web_fetch.rs          # Web Fetch 工具
│       └── calculator.rs         # 示例：计算器工具
├── ai_service.rs                 # 扩展支持 MCP
└── conversation.rs               # 扩展支持 tool 消息
```

### 3.2 核心类型定义

#### 3.2.1 统一的 Tool Trait

```rust
// src/mcp/tool_registry.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// 工具调用结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 工具定义（OpenAI 兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

/// 统一的工具接口
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具定义
    fn definition(&self) -> ToolDefinition;

    /// 执行工具
    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult>;

    /// 工具来源（内置/外部MCP）
    fn source(&self) -> ToolSource;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolSource {
    BuiltIn,              // 内置工具
    ExternalMcp(String),  // 外部 MCP 服务器名称
}
```

#### 3.2.2 Tool Registry

```rust
// src/mcp/tool_registry.rs
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // 自动注册内置工具
        registry.register_builtin_tools();

        registry
    }

    /// 注册内置工具
    fn register_builtin_tools(&mut self) {
        self.register(Arc::new(BuiltInTools::WebFetch));
        // 可以添加更多内置工具
    }

    /// 注册外部 MCP 工具
    pub fn register_mcp_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.definition().name.clone(), tool);
    }

    /// 获取所有工具定义（OpenAI 格式）
    pub fn to_openai_tools(&self) -> Vec<ChatCompletionTool> {
        self.tools.values().map(|tool| {
            let def = tool.definition();
            ChatCompletionTool::Function(FunctionObject {
                name: def.name,
                description: def.description,
                parameters: def.parameters,
            })
        }).collect()
    }

    /// 执行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolResult> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        tool.execute(arguments).await
    }
}
```

---

## 四、内置工具实现

### 4.1 Web Fetch 工具

#### 4.1.1 参数定义

```rust
// src/mcp/builtin/web_fetch.rs
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Web Fetch 工具参数
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WebFetchParams {
    /// 要获取的 URL
    pub url: String,

    /// 可选：CSS 选择器，提取特定内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// 可选：最大内容长度（字符）
    #[serde(default = "default_max_length")]
    pub max_length: usize,
}

fn default_max_length() -> usize { 10000 }
```

#### 4.1.2 工具实现

```rust
// src/mcp/builtin/web_fetch.rs
use super::*;
use async_trait::async_trait;

/// Web Fetch 工具实现
pub struct WebFetchTool {
    client: reqwest::Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("Aether-Matrix-Bot/1.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch content from a web URL and extract text. Useful for getting information from websites.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(WebFetchParams))
                .expect("Failed to generate schema"),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult> {
        let params: WebFetchParams = serde_json::from_value(arguments)
            .context("Invalid arguments for web_fetch")?;

        // 验证 URL
        let url = url::Url::parse(&params.url)
            .context("Invalid URL")?;

        // 获取网页内容
        let response = self.client
            .get(url.clone())
            .send()
            .await
            .context("Failed to fetch URL")?;

        if !response.status().is_success() {
            return Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some(format!("HTTP {}: {}", response.status(), params.url)),
            });
        }

        let html = response.text().await
            .context("Failed to read response body")?;

        // 提取文本内容
        let content = if let Some(selector) = &params.selector {
            self.extract_with_selector(&html, selector)?
        } else {
            self.extract_text(&html)
        };

        // 限制长度
        let truncated = if content.len() > params.max_length {
            format!("{}...\n\n[Truncated at {} characters]",
                &content[..params.max_length], params.max_length)
        } else {
            content
        };

        Ok(ToolResult {
            success: true,
            content: truncated,
            error: None,
        })
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
}

impl WebFetchTool {
    /// 从 HTML 中提取纯文本
    fn extract_text(&self, html: &str) -> String {
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(html, " ");

        text.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 使用 CSS 选择器提取内容
    fn extract_with_selector(&self, html: &str, selector: &str) -> Result<String> {
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse(selector)
            .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {:?}", e))?;

        let text = document.select(&selector)
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
```

---

## 五、AiService 集成

### 5.1 扩展 AiService 结构

```rust
// src/ai_service.rs
pub struct AiService {
    inner: Arc<AiServiceInner>,
    mcp_manager: Option<Arc<McpClientManager>>, // 可选，完全解耦
}

impl AiService {
    pub fn new(config: &Config) -> Self {
        let mcp_manager = if config.mcp_enabled {
            McpClientManager::from_config(&config.mcp_servers).ok()
        } else {
            None
        };

        Self {
            inner: Arc::new(AiServiceInner { ... }),
            mcp_manager,
        }
    }
}
```

### 5.2 带工具调用的聊天方法

```rust
// src/ai_service.rs
impl AiService {
    /// 带工具调用的聊天（支持内置+外部工具）
    pub async fn chat_with_tools(
        &self,
        session_id: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String> {
        // 1. 获取所有可用工具（内置 + 外部 MCP）
        let tools = self.get_all_tools().await;

        // 如果没有工具，直接普通对话
        if tools.is_empty() {
            return self.chat_with_system(session_id, prompt, system_prompt).await;
        }

        // 2. 调用 OpenAI with tools
        let response = self.call_openai_with_tools(
            session_id,
            prompt,
            system_prompt,
            &tools
        ).await?;

        // 3. 处理工具调用（如果有）
        if let Some(tool_calls) = response.tool_calls {
            for tool_call in tool_calls {
                // 执行工具（内置或外部）
                let result = self.execute_any_tool(&tool_call).await?;

                // 将结果添加到会话历史
                self.add_tool_message(session_id, &tool_call.id, result);
            }

            // 递归调用，让 AI 处理工具结果
            return self.chat_with_tools(session_id, "", system_prompt).await;
        }

        // 4. 返回最终回复
        Ok(response.content.unwrap_or_default())
    }

    /// 获取所有工具（内置 + 外部 MCP）
    async fn get_all_tools(&self) -> Vec<ChatCompletionTool> {
        let mut tools = Vec::new();

        // 添加内置工具
        if let Some(ref mcp_manager) = self.mcp_manager {
            tools.extend(mcp_manager.get_builtin_tools());
        }

        // 添加外部 MCP 工具
        if let Some(ref mcp_manager) = self.mcp_manager {
            if let Ok(external_tools) = mcp_manager.get_external_tools().await {
                tools.extend(external_tools);
            }
        }

        tools
    }

    /// 执行任意工具（内置或外部）
    async fn execute_any_tool(&self, tool_call: &ToolCall) -> Result<serde_json::Value> {
        let mcp_manager = self.mcp_manager.as_ref()
            .ok_or_else(|| anyhow::anyhow!("MCP not enabled"))?;

        let result = mcp_manager.execute_tool(
            &tool_call.function.name,
            tool_call.function.arguments.clone(),
        ).await?;

        // 返回结果（ToolResult -> JSON）
        Ok(serde_json::to_value(result)?)
    }
}
```

---

## 六、ConversationManager 扩展

### 6.1 支持工具消息

```rust
// src/conversation.rs
impl ConversationManager {
    /// 添加工具调用消息
    pub fn add_tool_call(&mut self, session_id: &str, tool_call: ToolCall) {
        let history = self.conversations
            .entry(session_id.to_string())
            .or_default();

        // 添加 assistant 的 tool_calls 消息
        history.push(ChatCompletionRequestMessage::Assistant(
            ChatCompletionRequestAssistantMessage {
                content: None,
                tool_calls: Some(vec![tool_call]),
                ..Default::default()
            }
        ));
    }

    /// 添加工具结果消息
    pub fn add_tool_result(
        &mut self,
        session_id: &str,
        tool_call_id: &str,
        result: serde_json::Value
    ) {
        let history = self.conversations
            .entry(session_id.to_string())
            .or_default();

        // 添加 tool 角色的消息
        history.push(ChatCompletionRequestMessage::Tool(
            ChatCompletionRequestToolMessage {
                content: serde_json::to_string(&result).unwrap_or_default(),
                tool_call_id: tool_call_id.to_string(),
            }
        ));
    }
}
```

---

## 七、配置设计

### 7.1 环境变量配置

```toml
# .env

# MCP 功能总开关
MCP_ENABLED=true

# 内置工具配置
MCP_BUILTIN_TOOLS_ENABLED=true
MCP_BUILTIN_WEB_FETCH_ENABLED=true
MCP_BUILTIN_WEB_FETCH_MAX_LENGTH=10000
MCP_BUILTIN_WEB_FETCH_TIMEOUT=10

# 外部 MCP 服务器配置（可选，JSON 数组）
MCP_EXTERNAL_SERVERS=[
  {
    "name": "filesystem",
    "transport": "stdio",
    "command": "mcp-server-filesystem",
    "args": ["--root", "/home/user"],
    "enabled": true
  },
  {
    "name": "database",
    "transport": "http",
    "url": "http://localhost:3000/mcp",
    "enabled": true
  }
]
```

### 7.2 配置结构体

```rust
// src/mcp/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub builtin_tools: BuiltinToolsConfig,
    pub external_servers: Vec<ExternalServerConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuiltinToolsConfig {
    pub enabled: bool,
    pub web_fetch: WebFetchConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebFetchConfig {
    pub enabled: bool,
    pub max_length: usize,
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalServerConfig {
    pub name: String,
    pub transport: TransportType,
    pub enabled: bool,

    // Stdio transport
    pub command: Option<String>,
    pub args: Option<Vec<String>>,

    // HTTP/SSE transport
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
    Http,
    Sse,
}
```

---

## 八、依赖项

### 8.1 Cargo.toml

```toml
[dependencies]
# MCP SDK
rmcp = { version = "1.1", features = ["client"] }
rmcp-macros = "1.1"

# 内置工具依赖
scraper = "0.22"           # HTML 解析（web_fetch）
regex = "1"                # 文本处理
url = "2.5"                # URL 解析

# 现有依赖保持不变
async-openai = { version = "0.33", features = ["chat-completion"] }
reqwest = { version = "0.12", features = ["rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = { version = "0.8", features = ["derive"] }
async-trait = "0.1"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

---

## 九、实现计划

### Phase 1: 基础架构（2天）

- [ ] 创建 `src/mcp/` 模块结构
- [ ] 实现统一的 `Tool` trait
- [ ] 实现 `ToolRegistry`
- [ ] 添加 MCP 配置支持
- [ ] 编写基础单元测试

**关键文件**：

- `src/mcp/mod.rs`
- `src/mcp/tool_registry.rs`
- `src/mcp/config.rs`

### Phase 2: 内置工具（2天）

- [ ] 实现 `web_fetch` 工具
- [ ] 实现工具注册机制
- [ ] 集成到 AiService
- [ ] 添加配置开关
- [ ] 编写工具测试

**关键文件**：

- `src/mcp/builtin/mod.rs`
- `src/mcp/builtin/web_fetch.rs`
- `src/ai_service.rs` (扩展)
- `src/conversation.rs` (扩展)

### Phase 3: 外部 MCP 集成（3天）

- [ ] 实现 Stdio transport
- [ ] 实现 HTTP transport
- [ ] 实现 SSE transport
- [ ] 实现外部工具注册
- [ ] 统一工具调用接口

**关键文件**：

- `src/mcp/client.rs`
- `src/mcp/transport/stdio.rs`
- `src/mcp/transport/http.rs`
- `src/mcp/transport/sse.rs`
- `src/mcp/tool_executor.rs`

### Phase 4: 测试和优化（2天）

- [ ] 单元测试（内置工具）
- [ ] 集成测试（混合工具调用）
- [ ] 错误处理优化
- [ ] 性能测试
- [ ] 文档完善

**测试文件**：

- `tests/mcp_integration.rs`
- `tests/builtin_tools_test.rs`

---

## 十、使用示例

### 10.1 基本使用

```rust
// 配置启用 MCP
let config = Config {
    mcp_enabled: true,
    builtin_tools: BuiltinToolsConfig {
        enabled: true,
        web_fetch: WebFetchConfig {
            enabled: true,
            max_length: 10000,
            timeout: 10,
        },
    },
    ..Default::default()
};

let ai_service = AiService::new(&config);

// AI 自动判断是否需要使用工具
let response = ai_service.chat_with_tools(
    "user-1",
    "帮我获取 https://example.com 的内容",
    None
).await?;

// AI 会自动调用 web_fetch 工具并返回结果
```

### 10.2 对话流程示例

```
用户: 帮我获取 https://example.com 的内容

AI 判断: 需要使用 web_fetch 工具

→ 调用工具: web_fetch({"url": "https://example.com"})
→ 内置 WebFetchTool 执行
← 返回结果: "Example Domain This domain is for use..."

AI 处理结果: 我已经获取了网站内容，这是一个示例域名...

→ 最终回复给用户
```

### 10.3 混合工具调用

```
用户: 帮我查询数据库用户数量，然后获取天气信息

AI 判断: 需要使用两个工具

→ 调用工具 1: database_query({"sql": "SELECT COUNT(*) FROM users"})
← 返回结果: {"count": 1234}

→ 调用工具 2: weather_fetch({"city": "Beijing"})
← 返回结果: {"temperature": 20, "condition": "sunny"}

AI 综合处理: 数据库有 1234 个用户，北京今天 20 度，天气晴朗...

→ 最终回复给用户
```

---

## 十一、后续扩展

### 11.1 更多内置工具

**计划添加**：

- `calculate` - 数学计算
- `search` - 搜索引擎集成
- `weather` - 天气查询
- `datetime` - 时间日期处理
- `json_parse` - JSON 解析和查询
- `file_read` - 文件读取（受权限控制）

### 11.2 高级特性

**计划实现**：

- 工具权限控制（RBAC）
- 工具调用统计和监控
- 工具结果缓存
- 流式工具调用
- 工具组合（Chain）
- 工具版本管理

### 11.3 MCP Server 功能

**未来扩展**：

- 暴露 Aether Bot 功能为 MCP tools
- 让其他 AI 应用调用 Matrix 功能
- 支持双向集成

---

## 十二、风险评估

| 风险           | 影响 | 概率 | 缓解措施                              |
| -------------- | ---- | ---- | ------------------------------------- |
| MCP SDK 不稳定 | 中   | 低   | 使用官方 SDK，版本锁定，充分测试      |
| Tool 调用超时  | 低   | 中   | 设置超时时间，异步执行，降级策略      |
| 性能影响       | 中   | 中   | 限制并发 tool 调用数量，结果缓存      |
| Token 消耗增加 | 中   | 高   | 缓存 tool 结果，优化 prompt，用户提醒 |
| 工具执行错误   | 低   | 中   | 完善错误处理，返回错误给 AI 重试      |
| 安全风险       | 高   | 低   | 工具权限控制，输入验证，沙箱隔离      |

---

## 十三、成功指标

### 13.1 功能指标

- ✅ 支持至少 3 个内置工具
- ✅ 支持 Stdio/HTTP/SSE 三种传输方式
- ✅ 工具调用成功率 > 95%
- ✅ 工具调用延迟 < 5s (P95)

### 13.2 质量指标

- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试覆盖核心场景
- ✅ 文档完善，示例清晰
- ✅ 错误处理健壮

### 13.3 用户体验

- ✅ 配置简单，开箱即用
- ✅ 降级优雅，不影响现有功能
- ✅ 错误提示友好
- ✅ 性能影响可控

---

## 十四、参考资料

### 14.1 官方文档

- [Model Context Protocol](https://modelcontextprotocol.io/)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)

### 14.2 SDK 和工具

- [rmcp - Rust MCP SDK](https://crates.io/crates/rmcp)
- [async-openai - Rust OpenAI SDK](https://crates.io/crates/async-openai)
- [MCP Servers](https://github.com/modelcontextprotocol/servers)

### 14.3 相关项目

- [Claude Desktop MCP Integration](https://docs.anthropic.com/claude/docs/mcp)
- [Cursor MCP Support](https://cursor.com/docs/context/mcp)

---

## 十五、变更日志

### v1.0 (2025-03-06)

- 初始方案设计
- 确定架构和模块结构
- 设计内置工具接口
- 规划实现步骤

---

**维护者**: Aether Team  
**最后更新**: 2025-03-06

