# Changelog

本文档记录项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.1.1] - 2026-03-05

### 新增

#### Phase 2 - Persona 人设模块
- 实现 `!persona list` 命令 - 列出所有可用的人设
- 实现 `!persona set <id>` 命令 - 设置当前房间的人设（需要房间管理员权限）
- 实现 `!persona off` 命令 - 关闭当前房间的人设（需要房间管理员权限）
- 实现 `!persona info <id>` 命令 - 查看人设详情
- 添加 4 个内置人设：
  - `sarcastic-dev` - 💻 毒舌程序员
  - `cyber-zen` - ☯️ 赛博禅师
  - `wiki-chan` - 📚 维基百科娘
  - `neko-chan` - 🐱 猫娘助手
- 集成 SQLite 数据库 (rusqlite)
- 创建数据库迁移系统
- 添加 `personas`、`room_persona`、`chat_history` 数据库表

#### Phase 1 - Admin 模块
- 实现 `!bot name <名称>` 命令 - 修改 Bot 显示名称（需要 Bot 所有者权限）
- 实现 `!bot avatar <url>` 命令 - 修改 Bot 头像（需要 Bot 所有者权限，支持 PNG/JPEG/GIF/WebP）
- 实现 `!bot join <room_id>` 命令 - 加入指定房间（需要 Bot 所有者权限）
- 实现 `!bot rooms` 命令 - 列出已加入的所有房间（需要 Bot 所有者权限）
- 实现 `!bot prefix <新前缀>` 命令 - 热更新命令前缀（需要 Bot 所有者权限）
- 实现 `!bot info` 命令 - 查看 Bot 基本信息
- 实现 `!bot leave` 命令 - 离开当前房间（需要房间管理员权限）
- 实现 `!ping` 命令 - 测试响应延迟

#### Phase 0 - 基础框架
- 命令系统基础结构
  - `CommandGateway` - 命令路由核心
  - `CommandContext` - 命令执行上下文
  - `Parser` - 命令解析器（支持引号包裹参数）
  - `Permission` - 权限模型（Anyone/RoomMod/BotOwner）
  - `CommandRegistry` - 命令处理器注册表
- UI 模块 - 毛玻璃风格消息模板
  - `info_card` - 信息卡片模板
  - `help_menu` - 帮助菜单模板
  - `success/error/warning` - 状态反馈模板
- 添加 `db_path` 配置项
- 添加 `bot_owners` 配置项
- 添加 `reqwest` 和 `mime` 依赖

#### Vision 多模态支持
- 支持图片理解功能，用户发送图片时自动分析图片内容
- 支持回复图片消息进行 Vision 分析
- 图片自动缩放至配置的最大尺寸（保持宽高比）
- 可配置的 Vision 专用模型和最大图片尺寸

#### 流式输出功能
- 支持流式响应，实现打字机效果
- 混合节流策略：时间触发 + 字符触发
- 首次发送新消息，后续使用 Matrix 消息编辑 API 更新
- 可配置的最小更新间隔和最小字符数

#### 配置系统
- 添加 .env 文件支持，从环境变量加载配置
- 添加设备ID配置支持（MATRIX_DEVICE_ID），避免重复登录创建新设备
- 添加可配置的设备显示名称（DEVICE_DISPLAY_NAME）
- 添加可配置的存储路径支持（STORE_PATH）
- 添加日志级别配置（LOG_LEVEL）
- 添加代理支持（PROXY）

#### 会话管理
- 支持提取引用消息内容作为上下文
- 多用户/多房间的会话历史隔离
- 可配置的最大历史轮数限制

#### MSC 3456 提及检测
- 支持检测 Matrix 房间提及事件
- 群聊中 @提及时自动响应

#### 测试基础设施
- 提取 AiServiceTrait trait 抽象，支持依赖注入和 mock 测试
- 添加 Config::from_env 单元测试
- 添加 RoomClient trait 和集成测试基础设施

### 改进

- 错误处理与同步稳定性优化
- Release 构建优化配置
  - 最高优化级别（opt-level = 3）
  - 链接时优化（LTO fat）
  - 单个代码生成单元，优化更彻底
  - panic = abort 减小二进制大小
  - 剥离符号信息

### 重构

- 提取 Bot 结构体，改进代码组织
- 移除 async-trait 依赖，Rust 2024 原生支持 trait async fn
- 使用 impl Future 语法优化 trait 定义
- 使用 watch 通道替代 broadcast 处理关闭信号
- 移除未使用的 RoomClient trait 和 SendMessageResult
- 移除测试模块中未使用的 MockAiService 代码
- 统一代码格式化风格
- `CommandGateway` 使用 `RwLock<Parser>` 支持命令前缀运行时热更新

### 文档

- 添加项目 README 文档
- 添加 CLAUDE.md 开发指南
- 为所有公开 API 添加详细文档注释
- 完善 CLAUDE.md 流式输出和配置说明
- 更新 README 反映 Vision 支持和最新架构
- 更新 CLAUDE.md 反映 Vision API 支持和最新代码结构
- 修复模块文档链接语法

### 依赖

- 升级 async-openai 至 0.33 并适配 API 变更
- 添加 `rusqlite` 依赖（SQLite 数据库）
- 添加 `reqwest` 和 `mime` 依赖
- 添加 `base64` 和 `image` 依赖（Vision API）

### 修复

- 修复 PersonaHandler 未注册到命令系统的问题
- 修复数据库连接与 matrix-sdk 的 sqlite 依赖冲突

### 变更

- 命令前缀从 `!ai` 改为 `!`

## [0.1.0] - 2026-02-XX

### 新增

- 初始化 matrix-xfy 项目
- 实现 Matrix AI 机器人核心功能
  - 基于 Matrix 协议的 AI 助手机器人
  - 使用 OpenAI 兼容 API 提供聊天功能
  - 支持私聊和群聊（命令前缀或 @提及）
  - 会话持久化