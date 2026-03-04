# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Matrix AI 机器人 - 一个基于 Matrix 协议的 AI 助手机器人，使用 OpenAI 兼容 API 提供聊天功能。

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
├── main.rs           # 入口点：初始化 Matrix 客户端、注册事件处理器、启动同步
├── config.rs         # 配置管理：从 .env 文件和环境变量加载配置
├── ai_service.rs     # AI 服务：封装 OpenAI API 调用，管理会话历史
├── event_handler.rs  # 事件处理：处理房间邀请和消息事件
└── conversation.rs   # 会话管理：多用户/多房间的会话历史管理
```

### 核心数据流

1. `main.rs` 初始化 Matrix 客户端并登录
2. 注册两类事件处理器：邀请事件（自动加入房间）和消息事件
3. 消息到达时 `EventHandler` 判断是否需要响应：
   - 私聊：总是响应
   - 群聊：需要命令前缀（默认 `!ai`）或 @提及
4. `AiService` 调用 OpenAI API 并管理会话历史
5. `ConversationManager` 按 session_id（用户ID或房间ID）隔离会话

### 关键类型

- `Config`: 所有配置项，从环境变量加载
- `AiService`: Clone 封装，内部使用 `Arc<AiServiceInner>` 共享状态
- `EventHandler`: Clone 封装，处理 Matrix 事件
- `ConversationManager`: 管理多会话历史，支持历史长度限制

## 配置

复制 `.env.example` 为 `.env` 并填写配置：

- `MATRIX_HOMESERVER` - Matrix 服务器地址
- `MATRIX_USERNAME` / `MATRIX_PASSWORD` - 登录凭据
- `OPENAI_API_KEY` - API 密钥
- `OPENAI_BASE_URL` - API 地址（支持兼容接口）
- `OPENAI_MODEL` - 模型名称（默认 gpt-4o-mini）
- `SYSTEM_PROMPT` - 系统提示词（可选）
- `BOT_COMMAND_PREFIX` - 命令前缀（默认 !ai）
- `MAX_HISTORY` - 最大历史轮数（默认 10）

## 机器人命令

- `!ai <消息>` - 与 AI 对话（前缀可配置）
- `!reset` - 清除当前会话历史
- `!help` - 显示帮助

## 代码风格

- 使用空格缩进（4 空格），Makefile 使用 tab
- 使用 `anyhow::Result` 进行错误处理
- 使用 `tracing` 进行日志记录