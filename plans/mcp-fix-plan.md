# MCP 功能完善修复方案

> **版本**: 1.0  
> **日期**: 2026-03-09  
> **目标**: 启用所有 MCP 功能，消除 ~1000+ 行 dead code  
> **预计工时**: 4小时

---

## 一、问题诊断

### 1.1 Dead Code 根本原因

**核心问题：MCP 命令处理器注册链断裂**

```rust
// src/event_handler.rs:182
if config.mcp.enabled {
    // TODO: 需要在AiService中添加获取mcp_server_manager的方法
    // command_gateway.register(Arc::new(McpHandler::new(ai_service.mcp_server_manager())));
}
```

这导致：
1. McpHandler 未注册到命令网关 → 用户无法使用 `!mcp` 命令
2. McpServerManager 无法被访问 → 外部 MCP 服务器无法连接
3. ToolRegistry 仅在内部使用 → 工具调用未激活

### 1.2 Dead Code 传播链

```
event_handler.rs:182 (被注释)
    ↓
modules/mcp/handlers.rs (189行 - 完全未使用)
    ↓
mcp/server_manager.rs (362行 - 完全未使用)
    ↓
mcp/transport/stdio.rs (62行 - 完全未使用)
    ↓
mcp/builtin/web_fetch.rs (部分未使用 - 实际执行路径未被调用)
```

**总 Dead Code：~1000+ 行（约 70% 的 MCP 代码）**

### 1.3 功能影响

| 功能 | 当前状态 | 影响 |
|------|----------|------|
| `!mcp list` | ❌ 不可用 | 用户无法查看可用工具 |
| `!mcp servers` | ❌ 不可用 | 用户无法查看服务器状态 |
| `!mcp reload` | ❌ 不可用 | 用户无法重载配置 |
| 外部 MCP 连接 | ❌ 不可用 | 无法连接外部 MCP 服务器 |
| 工具自动调用 | ⚠️ 未激活 | AI 无法自动调用工具 |
| Web Fetch 工具 | ⚠️ 部分可用 | 仅定义存在，未实际执行 |

---

## 二、修复方案总览

### 2.1 修复策略

**四步激活法：**

1. **添加公开 API** → 让 AiService 暴露 MCP 管理器
2. **注册命令处理器** → 取消注释，注册 McpHandler
3. **激活工具调用** → 在消息处理中集成工具调用
4. **验证与测试** → 确保所有功能正常工作

### 2.2 修复优先级

```
优先级 1 (关键路径):
├─ AiService 公开 API (必需)
└─ McpHandler 注册 (必需)

优先级 2 (功能激活):
├─ 工具调用集成 (可选，建议)
└─ HTTP/SSE 传输 (可选，后续)

优先级 3 (增强功能):
├─ 配置热重载 (后续)
└─ 更多内置工具 (后续)
```

---

## 三、详细修复步骤

### 步骤 1: 添加 AiService 公开 API

**目标：** 让外部能够访问 MCP 管理器和工具列表

#### 1.1 修改 `src/ai_service.rs`

**位置：** 第 434-438 行附近

```rust
impl AiService {
    /// 获取 MCP 服务器管理器
    ///
    /// # Returns
    ///
    /// 返回 MCP 服务器管理器的 Arc 引用（如果启用）
    pub fn mcp_server_manager(&self) -> Option<Arc<RwLock<McpServerManager>>> {
        self.inner.mcp_server_manager.clone()
    }
    
    /// 获取 MCP 工具注册表（如果启用）
    #[allow(dead_code)]
    pub fn inner_mcp_registry(&self) -> Option<Arc<RwLock<crate::mcp::ToolRegistry>>> {
        self.inner.mcp_registry.clone()
    }
    
    /// 列出所有可用的 MCP 工具
    ///
    /// # Returns
    ///
    /// 返回工具定义列表，如果 MCP 未启用则返回空列表
    pub async fn list_mcp_tools(&self) -> Vec<crate::mcp::ToolDefinition> {
        if let Some(ref registry) = self.inner.mcp_registry {
            let registry = registry.read().await;
            registry.to_openai_tools()
                .into_iter()
                .filter_map(|tool| match tool {
                    async_openai::types::chat::ChatCompletionTools::Function(f) => {
                        Some(crate::mcp::ToolDefinition {
                            name: f.function.name,
                            description: f.function.description.unwrap_or_default(),
                            parameters: f.function.parameters.unwrap_or(serde_json::Value::Null),
                        })
                    }
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
```

**变更：**
- ✅ 移除 `mcp_server_manager` 字段的 `#[allow(dead_code)]`
- ✅ 新增公开的 `mcp_server_manager()` getter 方法
- ✅ 新增 `list_mcp_tools()` 便捷方法
- ✅ 保留 `inner_mcp_registry()` (标记为 dead_code 但保留供测试使用)

#### 1.2 移除字段上的 dead_code 标记

**位置：** `src/ai_service.rs` 第 107-111 行

```rust
struct AiServiceInner {
    // ... 其他字段 ...
    
    /// MCP 工具注册表（可选）
    mcp_registry: Option<Arc<RwLock<crate::mcp::ToolRegistry>>>,
    /// MCP 服务器管理器（可选）
    mcp_server_manager: Option<Arc<RwLock<McpServerManager>>>,
}
```

**变更：** 删除 `#[allow(dead_code)]` 注解

---

### 步骤 2: 注册 MCP 命令处理器

**目标：** 激活 `!mcp` 命令，让用户能够管理 MCP 功能

#### 2.1 修改 `src/event_handler.rs`

**位置：** 第 179-183 行

```rust
// 注册MCP管理命令
if config.mcp.enabled {
    command_gateway.register(Arc::new(McpHandler::new(
        ai_service.mcp_server_manager(),
        Some(ai_service.clone()),
    )));
    info!("MCP 命令已注册，可用命令: !mcp list, !mcp servers, !mcp reload");
}
```

**变更：**
- ✅ 取消注释
- ✅ 调用新增的 `mcp_server_manager()` 方法
- ✅ 传递 `ai_service` 克隆（用于工具列表查询）
- ✅ 添加日志提示

#### 2.2 添加必要的导入

**位置：** `src/event_handler.rs` 文件顶部

```rust
use crate::modules::mcp::McpHandler;
use crate::mcp::McpServerManager;
```

---

### 步骤 3: 激活工具调用功能（可选但建议）

**目标：** 让 AI 能够自动判断并调用 MCP 工具

#### 3.1 在 EventHandler 中添加工具调用开关

**位置：** `src/event_handler.rs` 结构体定义部分

```rust
pub struct EventHandler<T: AiServiceTrait> {
    // ... 现有字段 ...
    /// 是否启用工具调用
    tools_enabled: bool,
}
```

**位置：** `EventHandler::new()` 方法中

```rust
impl<T: AiServiceTrait> EventHandler<T> {
    pub fn new(
        ai_service: T,
        bot_user_id: OwnedUserId,
        client: Client,
        config: &Config,
        persona_store: Option<PersonaStore>,
        muyu_store: Option<MuyuStore>,
    ) -> Self {
        // ... 现有代码 ...
        
        Self {
            // ... 现有字段 ...
            tools_enabled: config.mcp.enabled && config.mcp.builtin_tools.enabled,
        }
    }
}
```

#### 3.2 修改消息处理逻辑

**位置：** `handle_message()` 方法中的 AI 调用部分

**当前代码：**
```rust
// 使用流式聊天（带自定义系统提示词）
let (state, stream) = self.ai_service
    .chat_stream_with_system(&session_id, &message, persona_prompt)
    .await?;
```

**修改为：**
```rust
// 根据配置选择聊天模式
if self.tools_enabled {
    // 带工具调用的聊天（AI 可以自动调用工具）
    tracing::info!("使用工具调用模式");
    
    // TODO: 实现流式 + 工具调用的组合
    // 目前先使用非流式工具调用
    let response = self.ai_service
        .chat_with_tools(&session_id, &message, persona_prompt)
        .await?;
    
    // 直接发送完整响应
    let content = RoomMessageEventContent::text_plain(&response);
    room.send(content).await?;
    
    return Ok(());
} else {
    // 普通流式聊天
    let (state, stream) = self.ai_service
        .chat_stream_with_system(&session_id, &message, persona_prompt)
        .await?;
    
    // ... 现有的流式处理逻辑 ...
}
```

**注意：** 
- ⚠️ 这是一个简化实现，将流式输出改为一次性输出
- 🔄 后续可以优化为流式 + 工具调用的组合（需要更复杂的实现）
- ✅ 建议先验证功能，后续再优化体验

#### 3.3 添加配置选项（可选）

**位置：** `src/config.rs` 或 `config.toml`

```toml
[mcp]
enabled = true
tools_calling_enabled = true  # 是否启用自动工具调用
```

```rust
// src/mcp/config.rs
pub struct McpConfig {
    pub enabled: bool,
    pub tools_calling_enabled: bool,  // 新增
    // ...
}
```

---

### 步骤 4: 验证与测试

#### 4.1 编译检查

```bash
# 清理并重新编译
cargo clean
cargo build

# 检查 dead_code 警告
cargo clippy --all-targets --all-features -- -W dead_code
```

**预期结果：**
- ✅ 无 dead_code 警告（或大幅减少）
- ✅ McpHandler, McpServerManager 等被正确使用

#### 4.2 功能测试清单

**测试 1: 命令可用性**

```bash
# 在 Matrix 房间中测试
!mcp list         # 应显示可用工具列表
!mcp servers      # 应显示服务器连接状态
!mcp reload       # 应提示权限不足（非 BotOwner）
```

**测试 2: 工具调用**

```
用户: 帮我获取 https://example.com 的内容
预期: AI 自动调用 web_fetch 工具并返回结果
```

**测试 3: 外部 MCP 连接**

```toml
# config.toml
[[mcp.external_servers]]
name = "test-server"
transport = "stdio"
command = "echo"
args = ["test"]
enabled = true
```

```bash
# 启动 Bot 后检查日志
!mcp servers  # 应显示 test-server 连接状态
```

#### 4.3 性能测试

```bash
# 测试工具调用延迟
time curl -X POST http://localhost:8080/api/chat \
  -d '{"message": "获取 https://example.com 的内容"}'

# 预期: P95 < 2s
```

---

## 四、代码变更摘要

### 4.1 新增代码

| 文件 | 新增内容 | 行数 |
|------|----------|------|
| `src/ai_service.rs` | mcp_server_manager(), list_mcp_tools() | +20 行 |
| `src/event_handler.rs` | McpHandler 注册, 工具调用逻辑 | +15 行 |
| **总计** | | **+35 行** |

### 4.2 修改代码

| 文件 | 修改内容 | 影响 |
|------|----------|------|
| `src/ai_service.rs` | 移除 dead_code 标记 | 字段公开 |
| `src/event_handler.rs` | 取消注释 | 命令激活 |
| `src/event_handler.rs` | 添加工具调用路径 | 功能激活 |

### 4.3 消除的 Dead Code

| 文件 | Dead Code 行数 | 修复后状态 |
|------|----------------|-----------|
| `modules/mcp/handlers.rs` | 189 行 | ✅ 全部激活 |
| `mcp/server_manager.rs` | 362 行 | ✅ 全部激活 |
| `mcp/transport/stdio.rs` | 62 行 | ✅ 全部激活 |
| `mcp/builtin/web_fetch.rs` | ~150 行 | ✅ 部分激活 |
| `mcp/tool_registry.rs` | ~150 行 | ✅ 部分激活 |
| **总计** | **~1000 行** | **激活约 90%** |

---

## 五、后续优化建议

### 5.1 短期优化（1-2 天）

#### 优化 1: 流式输出 + 工具调用组合

**目标：** 支持流式输出同时保持工具调用能力

**方案：**
```rust
// 实现流式工具调用
pub async fn chat_stream_with_tools(
    &self,
    session_id: &str,
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<ChatStreamResponse> {
    // 1. 第一次调用判断是否需要工具
    let response = self.chat_with_tools_internal(session_id, prompt, system_prompt).await?;
    
    // 2. 如果有工具调用，执行并继续
    if let Some(tool_calls) = response.tool_calls {
        // 执行工具
        // ...
        // 递归调用
        return self.chat_stream_with_tools(session_id, "", system_prompt).await;
    }
    
    // 3. 如果是文本响应，返回流式
    let stream = self.create_stream_from_text(&response.content)?;
    Ok((state, stream))
}
```

**收益：** 提升用户体验，保持流式输出的打字机效果

#### 优化 2: 工具调用缓存

**目标：** 减少重复工具调用，节省 Token 和时间

**方案：**
```rust
pub struct ToolCache {
    cache: Arc<RwLock<LruCache<String, ToolResult>>>,
}

impl ToolCache {
    pub fn get_or_execute(&self, key: &str, executor: impl Future<Output = Result<ToolResult>>) -> Result<ToolResult> {
        // 检查缓存
        // 如果存在且未过期，返回缓存结果
        // 否则执行并缓存
    }
}
```

**收益：** 相同工具调用可节省 80%+ 时间

#### 优化 3: 工具调用统计与监控

**目标：** 了解工具使用情况，优化工具性能

**方案：**
```rust
pub struct ToolMetrics {
    pub tool_name: String,
    pub call_count: u64,
    pub success_count: u64,
    pub avg_duration_ms: f64,
    pub last_called: DateTime<Utc>,
}

// 在 McpHandler 中记录
impl McpHandler {
    async fn record_tool_call(&self, tool_name: &str, duration_ms: u64, success: bool) {
        // 记录到数据库或日志
    }
}
```

**命令扩展：**
```
!mcp stats          # 查看工具调用统计
!mcp stats <tool>   # 查看特定工具的统计
```

### 5.2 中期优化（3-5 天）

#### 优化 4: HTTP/SSE 传输实现

**目标：** 支持 HTTP/SSE 类型的 MCP 服务器

**实现文件：** `src/mcp/transport/http.rs`

```rust
pub struct HttpTransport {
    client: reqwest::Client,
    base_url: String,
}

impl HttpTransport {
    pub async fn new(config: &ExternalServerConfig) -> Result<Self> {
        // 实现 HTTP 连接
    }
    
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult> {
        // 实现工具调用
    }
}
```

**收益：** 可连接更多类型的 MCP 服务器（如远程 MCP 服务）

#### 优化 5: 配置热重载完善

**目标：** 无需重启即可更新 MCP 配置

**当前问题：** `server_manager.rs:358` 标记 TODO

```rust
pub async fn reload_config(&mut self, new_config: &McpConfig) -> Result<()> {
    // 1. 移除不再配置的服务器
    // 2. 添加新服务器
    // 3. 更新现有服务器配置
    // 4. 重新连接所有服务器
    // 5. 重新注册工具（需要区分内置/外部工具）
}
```

**命令：** `!mcp reload` (BotOwner 权限)

**收益：** 运维更灵活，无需重启 Bot

#### 优化 6: 工具权限控制

**目标：** 敏感工具需要特定权限才能调用

**方案：**
```rust
pub enum ToolPermission {
    Anyone,      // 任何人都可调用
    RoomMod,     // 房间管理员可调用
    BotOwner,    // 仅 Bot 所有者可调用
}

impl Tool trait {
    fn permission(&self) -> ToolPermission {
        ToolPermission::Anyone  // 默认
    }
}

// 在执行工具前检查权限
async fn execute_tool_with_permission_check(
    &self,
    tool_name: &str,
    args: Value,
    user_permission: &Permission,
) -> Result<ToolResult> {
    let tool = self.get_tool(tool_name)?;
    
    if !user_permission.can_call(&tool.permission()) {
        return Err(anyhow!("Permission denied for tool: {}", tool_name));
    }
    
    tool.execute(args).await
}
```

**示例配置：**
```toml
[tools.permissions]
file_delete = "bot_owner"    # 删除文件需要 BotOwner 权限
database_query = "room_mod"  # 数据库查询需要 RoomMod 权限
web_fetch = "anyone"         # 网页获取任何人都可以
```

**收益：** 安全性提升，防止工具滥用

### 5.3 长期优化（1-2 周）

#### 优化 7: 更多内置工具

**目标：** 提供更多开箱即用的工具

**工具列表：**

| 工具名称 | 功能 | 优先级 |
|---------|------|--------|
| `calculate` | 数学计算（使用 evalexpr） | 高 |
| `search` | 搜索引擎集成（DuckDuckGo API） | 高 |
| `weather` | 天气查询（OpenWeather API） | 中 |
| `datetime` | 时间日期处理和转换 | 中 |
| `json_parse` | JSON 解析和查询（jq 风格） | 低 |
| `qrcode` | 二维码生成 | 低 |

**实现模板：**
```rust
pub struct CalculateTool;

#[async_trait]
impl Tool for CalculateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "calculate".to_string(),
            description: "Evaluate mathematical expressions. Supports basic operations, functions, and variables.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(CalculateParams)).unwrap(),
        }
    }
    
    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult> {
        let params: CalculateParams = serde_json::from_value(arguments)?;
        let result = evalexpr::eval(&params.expression)?;
        
        Ok(ToolResult {
            success: true,
            content: result.to_string(),
            error: None,
        })
    }
    
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
}
```

#### 优化 8: 工具组合（Tool Chain）

**目标：** 支持工具之间的自动组合

**场景示例：**
```
用户: "查询北京天气并发送通知"
AI 判断:
  1. 调用 weather 工具获取天气
  2. 调用 notification 工具发送结果
```

**实现方案：**
```rust
pub struct ToolChain {
    steps: Vec<ToolStep>,
}

pub struct ToolStep {
    tool_name: String,
    input_mapping: HashMap<String, String>,  // 输入参数映射
    output_mapping: HashMap<String, String>, // 输出参数映射
}

impl ToolChain {
    pub async fn execute(&self, initial_input: Value) -> Result<Value> {
        let mut current_input = initial_input;
        
        for step in &self.steps {
            let input = self.map_input(&step.input_mapping, &current_input)?;
            let output = self.execute_tool(&step.tool_name, input).await?;
            current_input = self.map_output(&step.output_mapping, output)?;
        }
        
        Ok(current_input)
    }
}
```

**收益：** AI 可以完成更复杂的多步骤任务

#### 优化 9: MCP Server 功能

**目标：** 让 Aether Bot 成为 MCP Server，暴露功能给其他 AI 应用

**场景：** 让 Cursor/Claude Desktop 调用 Aether Bot 的 Matrix 功能

**实现：**
```rust
// src/mcp/server.rs
pub struct AetherMcpServer {
    matrix_client: Client,
}

impl AetherMcpServer {
    pub fn tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "send_matrix_message".to_string(),
                description: "Send a message to a Matrix room".to_string(),
                // ...
            },
            Tool {
                name: "get_room_members".to_string(),
                description: "Get members of a Matrix room".to_string(),
                // ...
            },
        ]
    }
}
```

**收益：** 双向集成，成为 MCP 生态的一部分

---

## 六、风险与缓解措施

### 6.1 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 工具调用失败影响用户体验 | 中 | 中 | 降级策略：失败时继续普通对话 |
| 工具调用延迟过高 | 中 | 中 | 超时控制：默认 10s，可配置 |
| Token 消耗大幅增加 | 低 | 高 | 工具结果缓存 + 结果截断 |
| 外部 MCP 服务器不稳定 | 中 | 中 | 重试机制 + 自动降级 |
| 工具权限控制不当 | 高 | 低 | 严格权限检查 + 审计日志 |

### 6.2 回滚方案

**如果修复后出现问题：**

1. **快速回滚：** 注释掉 `event_handler.rs:179-183` 的 MCP 注册代码
2. **配置禁用：** 设置 `MCP_ENABLED=false` 或 `mcp.enabled = false`
3. **降级运行：** 系统会自动降级为无 MCP 的普通模式

```bash
# 紧急禁用 MCP
export MCP_ENABLED=false
# 或修改 config.toml
[mcp]
enabled = false
```

---

## 七、验收标准

### 7.1 功能验收

- [ ] `!mcp list` 命令可用，正确显示工具列表
- [ ] `!mcp servers` 命令可用，正确显示服务器状态
- [ ] `!mcp reload` 命令可用，权限检查正确
- [ ] AI 可以自动判断并调用 `web_fetch` 工具
- [ ] 外部 MCP 服务器可以正确连接（Stdio 传输）
- [ ] 工具调用失败时系统优雅降级

### 7.2 代码质量验收

- [ ] Dead code 警告数量减少 > 80%
- [ ] 所有新增代码有适当的文档注释
- [ ] 单元测试覆盖率 > 70%
- [ ] 无 clippy 警告（或已知并接受的警告）

### 7.3 性能验收

- [ ] 工具调用 P95 延迟 < 2s
- [ ] 无 MCP 时性能无回归（基准测试对比）
- [ ] 内存占用增加 < 5MB

---

## 八、实施计划

### 8.1 时间安排

| 时间 | 任务 | 产出 |
|------|------|------|
| 第 1 小时 | 步骤 1: AiService API | 公开 getter 方法 |
| 第 2 小时 | 步骤 2: 命令注册 | McpHandler 激活 |
| 第 3 小时 | 步骤 3: 工具调用 | AI 工具调用路径 |
| 第 4 小时 | 步骤 4: 验证测试 | 测试报告 |

### 8.2 里程碑

- ✅ **M1**: AiService 公开 API 完成（第 1 小时）
- ✅ **M2**: MCP 命令可用（第 2 小时）
- ✅ **M3**: 工具调用激活（第 3 小时）
- ✅ **M4**: 所有测试通过（第 4 小时）

---

## 九、参考资料

### 9.1 相关文档

- [MCP 架构设计](./mcp.md)
- [MCP 外部服务器方案](./mcp-servers.md)
- [配置示例](../config.example.toml)

### 9.2 依赖文档

- [rmcp SDK 文档](https://docs.rs/rmcp/)
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [async-openai 文档](https://docs.rs/async-openai/)

---

## 十、变更日志

### v1.0 (2026-03-09)

- 初始方案设计
- 分析 dead code 根本原因
- 制定四步修复方案
- 规划后续优化路径

---

**维护者**: Aether Team  
**审核者**: (待指定)  
**最后更新**: 2026-03-09