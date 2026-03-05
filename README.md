# Aether Matrix

一个基于 Matrix 协议的 AI 助手机器人，使用 OpenAI 兼容 API 提供聊天功能。支持流式输出、多会话管理、会话持久化和图片理解。

## 功能特性

- 流式输出 - 打字机效果，实时显示 AI 响应
- 多会话管理 - 私聊按用户隔离，群聊按房间隔离
- 会话持久化 - 基于 SQLite 存储，重启后自动恢复
- 图片理解 - 支持 Vision API 理解用户发送或回复的图片
- 灵活配置 - 支持自定义命令前缀、系统提示词、历史长度
- 兼容性强 - 支持 OpenAI 及其兼容 API（如 DeepSeek、通义千问等）
- 可测试架构 - 使用 trait 抽象，支持 mock 测试

## 快速开始

### 安装

```bash
git clone https://github.com/your-username/aether-matrix.git
cd aether-matrix
make build
```

### 配置

复制配置模板并填写：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
# Matrix 配置
MATRIX_HOMESERVER=https://matrix.example.org
MATRIX_USERNAME=your_username
MATRIX_PASSWORD=your_password
MATRIX_DEVICE_ID=DEVICE_ID     # 可选，持久化设备

# OpenAI API 配置
OPENAI_API_KEY=your_api_key
OPENAI_BASE_URL=https://api.openai.com/v1
OPENAI_MODEL=gpt-4o-mini

# 可选配置
SYSTEM_PROMPT=你是一个有帮助的AI助手
BOT_COMMAND_PREFIX=!ai
MAX_HISTORY=10
```

### 运行

```bash
make run
```

## 使用方法

在 Matrix 客户端中与机器人交互：

私聊：直接发送消息即可

群聊：使用命令前缀或 @提及

```
!ai 你好
!reset    # 清除当前会话历史
!help     # 显示帮助
```

发送图片：机器人自动分析图片内容

回复图片：机器人分析被引用的图片

## 配置说明

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `MATRIX_HOMESERVER` | Matrix 服务器地址 | - |
| `MATRIX_USERNAME` | Matrix 用户名 | - |
| `MATRIX_PASSWORD` | Matrix 密码 | - |
| `MATRIX_DEVICE_ID` | 设备 ID（避免重复登录） | 随机生成 |
| `DEVICE_DISPLAY_NAME` | 设备显示名称 | AI Bot |
| `STORE_PATH` | Matrix SDK 存储路径 | ./store |
| `OPENAI_API_KEY` | API 密钥 | - |
| `OPENAI_BASE_URL` | API 地址 | https://api.openai.com/v1 |
| `OPENAI_MODEL` | 模型名称 | gpt-4o-mini |
| `SYSTEM_PROMPT` | 系统提示词 | - |
| `BOT_COMMAND_PREFIX` | 命令前缀 | !ai |
| `MAX_HISTORY` | 最大历史轮数 | 10 |
| `STREAMING_ENABLED` | 启用流式输出 | true |
| `STREAMING_MIN_INTERVAL_MS` | 流式更新最小间隔（毫秒） | 1000 |
| `STREAMING_MIN_CHARS` | 流式更新最小字符数 | 50 |
| `LOG_LEVEL` | 日志级别 | info |
| `VISION_ENABLED` | 启用图片理解 | true |
| `VISION_MODEL` | Vision 模型 | 使用 OPENAI_MODEL |
| `VISION_MAX_IMAGE_SIZE` | 图片最大尺寸（像素） | 1024 |

## 项目结构

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
└── event_handler/    # Matrix 事件处理模块
    ├── mod.rs        # 主处理器：消息路由和响应逻辑
    ├── invite.rs     # 邀请处理：自动接受房间邀请
    ├── streaming.rs  # 流式处理：打字机效果的节流逻辑
    └── extract.rs    # 消息提取：文本和引用图片提取

tests/                # 集成测试
├── ai_service_integration.rs
├── bot_integration.rs
└── event_handler_integration.rs
```

### 核心数据流

1. `main.rs` 加载配置并初始化日志
2. `Bot::new()` 创建 Matrix 客户端并登录
3. 注册两类事件处理器：邀请事件（自动加入房间）和消息事件
4. 消息到达时 `EventHandler` 判断是否需要响应：
   - 私聊：总是响应
   - 群聊：需要命令前缀（默认 `!ai`）或 @提及
5. 根据消息类型调用 `AiService`：
   - 文本消息：普通聊天
   - 图片消息：Vision API 分析
   - 回复图片：分析引用的图片
6. `ConversationManager` 按 session_id（用户ID或房间ID）隔离会话

### 流式输出机制

当 `STREAMING_ENABLED=true` 时，机器人使用流式响应：

1. `AiService::chat_stream()` 返回共享状态 `StreamingState` 和 Stream
2. `StreamingHandler` 消费 Stream，使用混合节流策略更新消息：
   - 时间触发：超过 `STREAMING_MIN_INTERVAL_MS`（默认 1000ms）
   - 字符触发：累积超过 `STREAMING_MIN_CHARS`（默认 50 字符）
3. 首次发送新消息，后续使用 Matrix 消息编辑 API 更新内容

### Vision API 支持

当 `VISION_ENABLED=true` 时，机器人支持图片理解：

1. 用户发送图片消息时，机器人下载并分析图片
2. 用户回复图片消息时，机器人分析引用的图片
3. 图片自动缩放至 `VISION_MAX_IMAGE_SIZE` 以下（保持宽高比）
4. 使用配置的 `VISION_MODEL` 或默认模型进行分析

## 开发

```bash
make build    # 编译项目（release）
make run      # 运行项目
make test     # 运行测试
make check    # 快速检查（不生成二进制文件）
make fmt      # 格式化代码
make lint     # 运行 clippy lint
make fix      # 自动修复代码问题并格式化
make clean    # 清理构建产物
```

## 技术栈

- [matrix-sdk](https://github.com/matrix-org/matrix-rust-sdk) - Matrix 客户端 SDK
- [async-openai](https://github.com/64bit/async-openai) - OpenAI 异步客户端
- [tokio](https://tokio.rs/) - 异步运行时
- [anyhow](https://github.com/dtolnay/anyhow) - 错误处理
- [image](https://github.com/image-rs/image) - 图片处理

## License

MIT