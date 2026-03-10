# Aether

基于 Matrix 协议的 AI 助手机器人，支持 OpenAI 兼容 API，具备流式输出、人设系统、权限控制和 MCP 集成。

## 功能特性

### 核心能力

- **流式输出** — 实时打字机效果，采用混合节流策略（时间 + 字符触发）
- **多会话管理** — 私聊按用户隔离，群聊按房间隔离
- **Vision API** — 图片理解，使用 Lanczos3 算法自动缩放
- **广泛兼容** — 支持 OpenAI、DeepSeek、通义千问等兼容 API
- **MCP 集成** — 内置工具（WebFetch）及外部 MCP 服务器支持

### 人设系统

- **4 个内置人设** — 毒舌程序员、赛博禅师、维基百科娘、猫娘助手
- **自定义人设** — 支持创建、删除和管理自定义人设
- **房间级配置** — 每个房间可独立设置人设
- **SQLite 持久化** — 所有人设和房间绑定永久存储

### 命令系统

- **模块化架构** — 基于 Trait 的命令处理器，职责清晰
- **三级权限** — 任何人、房间管理员、Bot 所有者
- **子命令支持** — 层级命令如 `!bot info`、`!persona set`
- **易于扩展** — 实现 `CommandHandler` trait 即可添加新命令

### 运维功能

- **运行时管理** — 无需重启即可修改名称、头像、加入房间
- **会话持久化** — SQLite 存储会话历史和配置
- **代理支持** — 支持 HTTP 和 SOCKS5 代理
- **可配置日志** — 通过环境变量调整日志级别

## 快速开始

### 安装

```bash
git clone https://github.com/your-username/aether.git
cd aether
make build
```

### 配置

复制配置模板：

```bash
cp .env.example .env
```

编辑 `.env` 文件填写配置：

```env
# Matrix 配置（必需）
MATRIX_HOMESERVER=https://matrix.example.org
MATRIX_USERNAME=your_username
MATRIX_PASSWORD=your_password

# OpenAI API 配置（必需）
OPENAI_API_KEY=your_api_key
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini

# 可选配置
BOT_OWNERS=@user1:matrix.org,@user2:matrix.org
MAX_HISTORY=10
STREAMING_ENABLED=true
VISION_ENABLED=true
MCP_ENABLED=true
MCP_BUILTIN_TOOLS_ENABLED=true
```

### 运行

```bash
make run
```

## MCP 配置详解

MCP (Model Context Protocol) 集成支持内置工具和外部 MCP 服务器。配置可以通过 `.env` 文件或 `config.toml` 文件进行。

### 环境变量配置

```env
# 启用 MCP 功能
MCP_ENABLED=true

# 启用内置工具
MCP_BUILTIN_TOOLS_ENABLED=true

# Web Fetch 工具配置
MCP_BUILTIN_WEB_FETCH_ENABLED=true
MCP_BUILTIN_WEB_FETCH_MAX_LENGTH=10000
MCP_BUILTIN_WEB_FETCH_TIMEOUT=10
```

### TOML 配置文件

创建 `config.toml` 文件以配置更复杂的 MCP 设置：

```toml
[mcp]
enabled = true

[mcp.builtin_tools]
enabled = true

[mcp.builtin_tools.web_fetch]
enabled = true
max_length = 10000
timeout = 10

[[mcp.external_servers]]
name = "filesystem"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/user/documents"]
enabled = true

[[mcp.external_servers]]
name = "database"
transport = "stdio"  
command = "mcp-server-sqlite"
args = ["--database", "/path/to/database.db"]
enabled = true
```

### 外部 MCP 服务器支持

当前支持的传输类型：
- **stdio**: 通过标准输入输出通信（推荐）
- **HTTP/SSE**: 通过 HTTP API 通信（需要 rmcp 1.1+，当前版本暂不支持）

注意：HTTP/SSE 传输在当前版本中不可用，因为 rmcp 1.0.0 不支持这些功能。

### MCP 使用示例

#### 基本工具调用

当 MCP 工具可用时，AI 会自动决定是否调用工具：

```
用户: ! 获取 https://example.com 的内容
AI: [自动调用 web_fetch 工具获取网页内容并返回摘要]
```

#### 查看可用工具

使用 `!mcp list` 命令查看所有可用的 MCP 工具：

```
!mcp list
```

#### 管理 MCP 服务器

使用 `!mcp servers` 查看外部 MCP 服务器连接状态：

```
!mcp servers
```

只有 Bot 所有者可以重载 MCP 配置：

```
!mcp reload
```

## 命令参考

### 聊天命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!<消息>` | 与 AI 对话 | 任何人 |
| `!reset` | 清除当前会话历史 | 任何人 |
| `!help` | 显示帮助菜单 | 任何人 |

### 人设命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!persona list` | 列出所有人设 | 任何人 |
| `!persona info <id>` | 查看人设详情 | 任何人 |
| `!persona set <id>` | 设置房间人设 | 房间管理员 |
| `!persona off` | 关闭房间人设 | 房间管理员 |
| `!persona create <id> "<名称>" "<提示词>"` | 创建自定义人设 | 房间管理员 |
| `!persona delete <id>` | 删除自定义人设 | 房间管理员 |

### Bot 管理命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!bot info` | 查看 Bot 基本信息 | 任何人 |
| `!bot ping` | 测试响应延迟 | 任何人 |
| `!bot name <名称>` | 修改显示名称 | Bot 所有者 |
| `!bot avatar <url>` | 修改头像 URL | Bot 所有者 |
| `!bot join <room_id>` | 加入指定房间 | Bot 所有者 |
| `!bot rooms` | 列出已加入房间 | Bot 所有者 |
| `!leave` | 离开当前房间 | 房间管理员 |

### 赛博木鱼命令（彩蛋）

| 命令 | 说明 | 权限 |
|------|------|------|
| `!木鱼` | 敲击木鱼，积累功德 | 任何人 |
| `!功德` | 查看个人功德信息 | 任何人 |
| `!功德榜` | 查看房间功德排行榜 | 任何人 |
| `!称号 [名称]` | 查看或装备称号 | 任何人 |
| `!背包` | 查看物品背包 | 任何人 |

### MCP 命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `!mcp list` | 列出可用的 MCP 工具 | 任何人 |
| `!mcp servers` | 查看 MCP 服务器状态 | 任何人 |
| `!mcp reload` | 重载 MCP 配置 | Bot 所有者 |

## 内置人设

| ID | 名称 | 描述 |
|----|------|------|
| `sarcastic-dev` | 毒舌程序员 | 20 年经验老程序员，对低质量代码愤怒，先吐槽再回答 |
| `cyber-zen` | 赛博禅师 | 用 TCP/IP 诠释佛法，简短而深邃 |
| `wiki-chan` | 维基百科娘 | 知识渊博、严谨客观、标注来源 |
| `neko-chan` | 猫娘助手 | 语气活泼可爱，句末加「喵~」 |

### 示例：自定义人设

```bash
# 创建自定义人设
!persona create rust-mentor "Rust 导师" "你是一位经验丰富的 Rust 开发者，帮助他人编写安全、地道的 Rust 代码。"

# 设置到当前房间
!persona set rust-mentor

# 查看详情
!persona info rust-mentor

# 完成后删除
!persona delete rust-mentor
```

## 权限模型

| 级别 | 描述 | 适用命令 |
|------|------|----------|
| **任何人** | 任何房间成员 | 聊天、查看人设、ping |
| **房间管理员** | 房间管理员（power_level >= 50）或私聊用户 | 设置人设、离开房间 |
| **Bot 所有者** | Bot 所有者（通过 `BOT_OWNERS` 配置） | 修改名称/头像、加入房间、重载 MCP |

**注意：** 私聊房间默认赋予用户房间管理员权限。

## 配置参考

### 必需配置

| 配置项 | 说明 |
|--------|------|
| `MATRIX_HOMESERVER` | Matrix 服务器地址 |
| `MATRIX_USERNAME` | Matrix 用户名 |
| `MATRIX_PASSWORD` | Matrix 密码 |
| `OPENAI_API_KEY` | OpenAI API 密钥 |

### 可选配置

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `MATRIX_DEVICE_ID` | 持久化设备 ID（避免重复登录） | 自动生成 |
| `DEVICE_DISPLAY_NAME` | 设备显示名称 | "AI Bot" |
| `STORE_PATH` | Matrix SDK 存储路径 | `./store` |
| `OPENAI_BASE_URL` | API 基础地址 | `https://api.openai.com/v1` |
| `OPENAI_MODEL` | 模型名称 | `gpt-4o-mini` |
| `SYSTEM_PROMPT` | 全局系统提示词 | — |
| `BOT_COMMAND_PREFIX` | 命令前缀 | `!` |
| `BOT_OWNERS` | Bot 所有者列表（逗号分隔） | — |
| `DB_PATH` | 数据库路径 | `./data/aether.db` |
| `MAX_HISTORY` | 最大对话轮数 | `10` |
| `STREAMING_ENABLED` | 启用流式输出 | `true` |
| `STREAMING_MIN_INTERVAL_MS` | 流式更新最小间隔（毫秒） | `1000` |
| `STREAMING_MIN_CHARS` | 流式更新最小字符数 | `50` |
| `VISION_ENABLED` | 启用图片理解 | `true` |
| `VISION_MODEL` | Vision 模型名称 | 使用 `OPENAI_MODEL` |
| `VISION_MAX_IMAGE_SIZE` | 图片最大边长（像素） | `1024` |
| `PROXY` | HTTP/SOCKS5 代理 URL | — |
| `LOG_LEVEL` | 日志级别 | `info` |
| `MCP_ENABLED` | 启用 MCP 集成 | `true` |
| `MCP_BUILTIN_TOOLS_ENABLED` | 启用内置 MCP 工具 | `true` |
| `PROXY` | HTTP/SOCKS5 代理 URL | — |
| `LOG_LEVEL` | 日志级别 | `info` |
| `MCP_ENABLED` | 启用 MCP 集成 | `true` |
| `MCP_BUILTIN_TOOLS_ENABLED` | 启用内置 MCP 工具 | `true` |

## 架构设计

### 模块结构

```
src/
├── main.rs              # 入口点
├── lib.rs               # 库导出
├── bot.rs               # Bot 初始化
├── config.rs            # 配置管理（7 个配置组）
├── traits.rs            # 核心 trait（AiServiceTrait）
├── ai_service.rs        # OpenAI API 封装
├── conversation.rs      # 会话管理
├── event_handler.rs     # Matrix 事件处理
├── media.rs             # 图片处理
├── command/             # 命令系统
│   ├── gateway.rs       # 命令路由
│   ├── registry.rs      # CommandHandler trait
│   ├── permission.rs    # 权限模型
│   ├── parser.rs        # 命令解析
│   └── context.rs       # 执行上下文
├── modules/             # 功能模块
│   ├── admin/           # Bot 管理
│   ├── persona/         # 人设管理
│   ├── mcp/             # MCP 命令
│   └── muyu/            # 赛博木鱼
├── mcp/                 # MCP 集成
│   ├── config.rs        # MCP 配置
│   ├── tool_registry.rs # 工具注册表
│   ├── server_manager.rs# 服务器管理
│   ├── builtin/         # 内置工具
│   └── transport/       # 传输层
├── store/               # 数据持久化
│   ├── database.rs      # SQLite + 迁移
│   └── persona_store.rs # 人设存储
└── ui/                  # UI 模板
    └── templates.rs     # HTML 消息模板

tests/                   # 集成测试
migrations/              # 数据库迁移
```

### 命令系统

命令系统采用基于 Trait 的架构：

```rust
#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str { "" }
    fn permission(&self) -> Permission { Permission::Anyone }
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()>;
}
```

**核心组件：**

- **CommandRegistry** — 管理所有命令处理器
- **CommandGateway** — 路由消息到处理器
- **CommandContext** — 封装房间、发送者、参数等上下文
- **Permission** — 三级访问控制

### 数据流

```
Matrix 消息
    │
    ▼
EventHandler（检查：私聊 / 前缀 / @提及）
    │
    ▼
CommandGateway（解析命令名和参数）
    │
    ▼
CommandRegistry.get(name)
    │
    ▼
Permission.check()
    │
    ▼
CommandHandler.execute(ctx)
    │
    ▼
AiService / PersonaStore / Bot API
    │
    ▼
Room.send() — 发送响应
```

### 流式输出机制

启用流式输出（`STREAMING_ENABLED=true`）时：

1. **混合节流** — 满足以下任一条件即触发更新：
   - 时间：超过 `STREAMING_MIN_INTERVAL_MS`（默认 1000ms）
   - 字符：累积超过 `STREAMING_MIN_CHARS`（默认 50 字符）

2. **消息更新流程**：
   - 首个片段：发送新消息
   - 后续片段：使用 Matrix 替换 API 编辑消息
   - 流结束：发送最终版本

3. **自动保存** — 完整响应自动保存到会话历史

### Vision 处理流程

图片处理工作流：

1. **下载** — 通过 Matrix SDK 获取图片（支持缓存）
2. **缩放** — Lanczos3 算法，保持宽高比，最大 `VISION_MAX_IMAGE_SIZE`
3. **编码** — 转换为 base64 data URL（PNG 格式）
4. **分析** — 发送至 Vision API 进行理解

### 人设系统设计

- **PersonaStore** — 基于 SQLite 的 CRUD，使用 UPSERT 保证初始化幂等性
- **房间绑定** — `room_persona` 表关联房间与人设
- **自动重命名** — 设置人设时自动更新 Bot 显示名称
- **内置人设** — 启动时初始化，不可删除

## 开发

```bash
make build    # 编译 release 版本
make run      # 运行 Bot
make test     # 运行测试
make check    # 快速检查（不生成二进制）
make fmt      # 格式化代码
make lint     # 运行 clippy
make fix      # 自动修复并格式化
make clean    # 清理构建产物
```

### 运行测试

```bash
# 运行所有测试
make test

# 运行指定测试
cargo test ai_service

# 显示输出
cargo test -- --nocapture
```

## 技术栈

| 类别 | 技术 |
|------|------|
| **Matrix SDK** | [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) |
| **AI 客户端** | [async-openai](https://github.com/64bit/async-openai) |
| **异步运行时** | [tokio](https://tokio.rs/) |
| **数据库** | [rusqlite](https://github.com/rusqlite/rusqlite) |
| **图片处理** | [image](https://github.com/image-rs/image) |
| **HTTP 客户端** | [reqwest](https://github.com/seanmonstar/reqwest) |
| **MCP SDK** | [rmcp](https://crates.io/crates/rmcp) |
| **错误处理** | [anyhow](https://github.com/dtolnay/anyhow), [thiserror](https://github.com/dtolnay/thiserror) |
| **异步 Trait** | [async-trait](https://github.com/dtolnay/async-trait) |

## 数据库模式

### 人设表

```sql
CREATE TABLE personas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    avatar_emoji TEXT,
    is_builtin INTEGER DEFAULT 0,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE room_persona (
    room_id TEXT PRIMARY KEY,
    persona_id TEXT REFERENCES personas(id) ON DELETE CASCADE,
    enabled INTEGER DEFAULT 1,
    set_by TEXT,
    set_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 聊天记录表

```sql
CREATE TABLE chat_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## 许可证

MIT
