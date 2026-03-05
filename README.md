# Aether Matrix

一个基于 Matrix 协议的 AI 助手机器人，支持 OpenAI 兼容 API、流式输出和会话管理。

## 功能特性

- **流式输出** - 打字机效果，实时显示 AI 响应
- **多会话管理** - 私聊按用户隔离，群聊按房间隔离
- **会话持久化** - 基于 SQLite 存储，重启后自动恢复
- **灵活配置** - 支持自定义命令前缀、系统提示词、历史长度
- **兼容性强** - 支持 OpenAI 及其兼容 API（如 DeepSeek、通义千问等）

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

**私聊** - 直接发送消息即可

**群聊** - 使用命令前缀或 @提及

```
!ai 你好
!reset    # 清除当前会话历史
!help     # 显示帮助
```

## 配置说明

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `MATRIX_HOMESERVER` | Matrix 服务器地址 | - |
| `MATRIX_USERNAME` | Matrix 用户名 | - |
| `MATRIX_PASSWORD` | Matrix 密码 | - |
| `MATRIX_DEVICE_ID` | 设备 ID（避免重复登录） | 随机生成 |
| `OPENAI_API_KEY` | API 密钥 | - |
| `OPENAI_BASE_URL` | API 地址 | `https://api.openai.com/v1` |
| `OPENAI_MODEL` | 模型名称 | `gpt-4o-mini` |
| `SYSTEM_PROMPT` | 系统提示词 | - |
| `BOT_COMMAND_PREFIX` | 命令前缀 | `!ai` |
| `MAX_HISTORY` | 最大历史轮数 | `10` |
| `STREAMING_ENABLED` | 启用流式输出 | `true` |
| `STREAMING_MIN_INTERVAL_MS` | 流式更新最小间隔 | `1000` |
| `STREAMING_MIN_CHARS` | 流式更新最小字符数 | `50` |
| `LOG_LEVEL` | 日志级别 | `info` |

## 项目结构

```
src/
├── main.rs           # 入口点
├── bot.rs            # Bot 结构体，封装初始化和运行逻辑
├── config.rs         # 配置管理
├── ai_service.rs     # AI 服务封装
├── conversation.rs   # 会话管理
└── event_handler.rs  # Matrix 事件处理
```

## 开发

```bash
make check    # 快速检查
make test     # 运行测试
make lint     # 代码检查
make fix      # 自动修复
```

## 技术栈

- [matrix-sdk](https://github.com/matrix-org/matrix-rust-sdk) - Matrix 客户端 SDK
- [async-openai](https://github.com/64bit/async-openai) - OpenAI 异步客户端
- [tokio](https://tokio.rs/) - 异步运行时
- [anyhow](https://github.com/dtolnay/anyhow) - 错误处理

## License

MIT