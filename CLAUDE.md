# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Aether Matrix Bot - 一个基于 Matrix 协议的 AI 助手机器人，使用 OpenAI 兼容 API 提供聊天功能。支持流式输出、多会话管理、图片理解、Persona 人设系统、权限控制。

## 常用命令

```bash
make build    # 编译项目 (release)
make run      # 运行项目
make test     # 运行测试
make check    # 快速检查（不生成二进制文件）
make fmt      # 格式化代码
make lint     # 运行 clippy lint
make fix      # 自动修复代码问题并格式化
```

## 架构

```
src/
├── main.rs           # 入口点：初始化日志和 Bot
├── lib.rs            # 库入口：模块导出和文档
├── bot.rs            # Bot 结构体：初始化 Matrix 客户端和事件处理器
├── config.rs         # 配置管理：从环境变量加载配置
├── ai_service.rs     # AI 服务：封装 OpenAI API，管理会话历史
├── conversation.rs   # 会话管理：多用户/多房间的会话历史管理
├── traits.rs         # Trait 抽象：AiServiceTrait 支持 mock 测试
├── media.rs          # 媒体处理：图片下载、缩放、base64 编码
├── event_handler.rs  # Matrix 事件处理：消息路由和响应逻辑
├── command/          # 命令系统模块
│   ├── mod.rs        # 模块入口
│   ├── context.rs    # 命令执行上下文
│   ├── gateway.rs    # 命令网关：路由分发
│   ├── parser.rs     # 命令解析
│   ├── permission.rs # 权限控制：三级权限模型
│   └── registry.rs   # 命令处理器注册表
├── modules/          # 功能模块
│   ├── admin/        # Bot 管理命令（bot, leave, ping）
│   └── persona/      # 人设管理命令
├── store/            # 数据存储
│   ├── database.rs   # SQLite 数据库连接
│   └── persona_store.rs  # Persona 存储
└── ui/               # UI 模块
    └── templates.rs  # HTML 消息模板（卡片风格）
```

### 核心数据流

1. `main.rs` 加载配置并初始化日志
2. `Bot::new()` 创建 Matrix 客户端并登录，初始化数据库和 Persona
3. 注册事件处理器：邀请事件（自动加入房间）和消息事件
4. 消息到达时 `EventHandler` 判断是否需要响应：
   - 私聊：总是响应
   - 群聊：需要命令前缀（默认 `!`）或 @提及
5. 命令消息通过 `CommandGateway` 路由到对应处理器
6. 非命令消息调用 `AiService`：
   - 检查房间是否设置了 Persona，应用对应系统提示词
   - 文本消息：普通聊天
   - 图片消息：Vision API 分析
7. `ConversationManager` 按 session_id（用户ID或房间ID）隔离会话

### 关键类型

- `Config`: 所有配置项，从环境变量加载
- `Bot`: 机器人主结构，封装 Matrix 客户端初始化和事件处理器注册
- `AiService`: Clone 封装，内部使用 `Arc<AiServiceInner>` 共享状态，实现 `AiServiceTrait`
- `AiServiceTrait`: AI 服务 trait 抽象，支持依赖注入和 mock 测试
- `EventHandler<T>`: 泛型事件处理器，支持任意实现 `AiServiceTrait` 的服务
- `CommandGateway`: 命令网关，负责命令路由和权限检查
- `CommandHandler`: 命令处理器 trait，定义命令的执行接口
- `Permission`: 三级权限模型（Anyone, RoomMod, BotOwner）
- `Persona`: 人设定义，包含系统提示词、显示名称、头像 emoji
- `PersonaStore`: 人设存储，管理内置和自定义人设

### 权限系统

三级权限模型：
- `Anyone`: 任何房间成员都可以执行
- `RoomMod`: 房间管理员（power_level >= 50），私聊房间自动拥有此权限
- `BotOwner`: Bot 所有者，通过 `BOT_OWNERS` 环境变量配置

### Persona 人设系统

每个房间可以设置独立的 Persona，影响 AI 的系统提示词和响应风格。
内置人设：
- `sarcastic-dev`: 毒舌程序员
- `cyber-zen`: 赛博禅师
- `wiki-chan`: 维基百科娘
- `neko-chan`: 猫娘助手

### 流式输出机制

当 `STREAMING_ENABLED=true` 时，机器人使用流式响应：
1. `AiService::chat_stream()` 返回共享状态 `StreamingState` 和 Stream
2. 消费 Stream，使用混合节流策略更新消息：
   - 时间触发：超过 `STREAMING_MIN_INTERVAL_MS`（默认 1000ms）
   - 字符触发：累积超过 `STREAMING_MIN_CHARS`（默认 50 字符）
3. 首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容

### Vision API 支持

当 `VISION_ENABLED=true` 时，机器人支持图片理解：
1. 用户发送图片消息时，机器人下载并分析图片
2. 用户回复图片消息时，机器人分析引用的图片
3. 图片自动缩放至 `VISION_MAX_IMAGE_SIZE` 以下（保持宽高比）

## 配置

复制 `.env.example` 为 `.env` 并填写配置：

**必需配置：**
- `MATRIX_HOMESERVER` - Matrix 服务器地址
- `MATRIX_USERNAME` / `MATRIX_PASSWORD` - 登录凭据
- `OPENAI_API_KEY` - API 密钥

**可选配置：**
- `MATRIX_DEVICE_ID` - 设备ID（持久化设备，避免重复登录创建新设备）
- `DEVICE_DISPLAY_NAME` - 设备显示名称（默认 AI Bot）
- `STORE_PATH` - Matrix SDK 存储路径（默认 ./store）
- `OPENAI_BASE_URL` - API 地址（默认 OpenAI，支持兼容接口）
- `OPENAI_MODEL` - 模型名称（默认 gpt-4o-mini）
- `SYSTEM_PROMPT` - 系统提示词
- `BOT_COMMAND_PREFIX` - 命令前缀（默认 !）
- `BOT_OWNERS` - Bot 所有者列表（逗号分隔的 Matrix 用户 ID）
- `MAX_HISTORY` - 最大历史轮数（默认 10）
- `DB_PATH` - 数据库文件路径（默认 ./data/aether.db）
- `STREAMING_ENABLED` - 启用流式输出（默认 true）
- `STREAMING_MIN_INTERVAL_MS` - 流式更新最小间隔（默认 1000）
- `STREAMING_MIN_CHARS` - 流式更新最小字符数（默认 50）
- `LOG_LEVEL` - 日志级别（默认 info）
- `VISION_ENABLED` - 启用图片理解（默认 true）
- `VISION_MODEL` - Vision 模型（默认使用 OPENAI_MODEL）
- `VISION_MAX_IMAGE_SIZE` - 图片最大尺寸（默认 1024）
- `PROXY` - HTTP 代理 URL（可选）

## 机器人命令

**Bot 管理命令：**
- `!bot info` - 查看 Bot 基本信息
- `!bot ping` - 测试响应延迟
- `!bot name <名称>` - 修改显示名称（Bot 所有者）
- `!bot avatar <url>` - 修改头像（Bot 所有者）
- `!bot join <房间ID>` - 加入房间（Bot 所有者）
- `!bot rooms` - 列出已加入房间（Bot 所有者）
- `!leave` - 离开当前房间（房间管理员）
- `!ping` - 测试响应

**Persona 命令：**
- `!persona list` - 列出所有人设
- `!persona set <id>` - 设置房间人设（房间管理员）
- `!persona off` - 关闭房间人设（房间管理员）
- `!persona info <id>` - 查看人设详情
- `!persona create <id> "<名称>" "<提示词>"` - 创建自定义人设（房间管理员）
- `!persona delete <id>` - 删除自定义人设（房间管理员）

**其他：**
- `!<消息>` - 与 AI 对话
- `!reset` - 清除当前会话历史
- 发送图片 - 分析图片内容
- 回复图片 - 分析引用的图片

## 代码风格

- 使用空格缩进（4 空格），Makefile 使用 tab
- 使用 `anyhow::Result` 进行错误处理
- 使用 `tracing` 进行日志记录
- 使用 `Arc` + `RwLock`/`Mutex` 模式共享可变状态
- Rust edition 2024
- 使用 trait 抽象（`AiServiceTrait`, `CommandHandler`）支持测试

## 运行时数据

- `./store/` - Matrix SDK 的 SQLite 存储目录，保存会话和同步状态
- `./data/aether.db` - Bot 数据库，存储 Persona 等持久化数据