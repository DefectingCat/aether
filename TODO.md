# Aether Matrix Bot — 任务清单

## 当前状态

### 已完成
- [x] 基础项目结构
- [x] Matrix 客户端连接与 session 持久化
- [x] 邀请自动接受
- [x] 消息处理（命令前缀、@提及、MSC3456 mentions）
- [x] AI 对话（OpenAI 兼容 API）
- [x] 流式响应（打字机效果）
- [x] 会话历史管理（内存存储）
- [x] 代理支持
- [x] 单元测试框架
- [x] 命令系统基础结构
- [x] bot_owners 配置项
- [x] **命令界面毛玻璃风格模板**（2026-03-05）

---

## Phase 0 · 基础框架完善

### 0.1 命令系统 ✅
- [x] `src/command/mod.rs` - 命令模块入口
- [x] `src/command/context.rs` - CommandContext 结构体
- [x] `src/command/parser.rs` - 命令解析器
  - [x] 前缀解析
  - [x] 支持引号包裹参数
- [x] `src/command/permission.rs` - 权限模型
- [x] `src/command/registry.rs` - Handler 注册表
- [x] `src/command/gateway.rs` - CommandGateway 路由核心

### 0.2 UI 模块 ✅
- [x] `src/ui/mod.rs` - UI 模块入口
- [x] `src/ui/templates.rs` - 毛玻璃风格消息模板
  - [x] 信息卡片模板 (info_card)
  - [x] 帮助菜单模板 (help_menu)
  - [x] 排行榜模板 (leaderboard)
  - [x] 状态反馈模板 (success/error/warning/info)
  - [x] 子命令列表模板 (subcommand_list)

### 0.3 配置扩展
- [x] 添加 `bot_owners` 配置项
- [x] 添加 `db_path` 配置项

### 0.4 数据库层
- [x] SQLite 集成
- [x] migrations 目录

### 0.5 错误处理
- [ ] 自定义错误类型
- [ ] 自动重连机制

---

## Phase 1 · Admin 模块

### 指令实现
- [x] `!bot name <名称>` - 修改 Bot 显示名称
- [x] `!bot avatar <url>` - 修改 Bot 头像
- [x] `!bot info` - 使用毛玻璃风格卡片
- [x] `!bot join <room_id>` - 加入指定房间
- [x] `!bot leave` - 使用毛玻璃风格提示
- [x] `!bot rooms` - 列出已加入房间
- [x] `!bot prefix <新前缀>` - 热更新命令前缀
- [x] `!ping` - 使用毛玻璃风格提示

---

## Phase 2 · Persona 人设模块

### 数据库设计
- [x] personas 表
- [x] room_persona 表
- [x] chat_history 表

### 功能实现
- [x] `!persona set <id>`
- [ ] `!persona create <id>`
- [x] `!persona list`
- [x] `!persona off`
- [x] `!persona info <id>`

### 内置人设
- [x] 毒舌程序员
- [x] 赛博禅师
- [x] 维基百科娘
- [x] 猫娘助手

---

## Phase 3 · 梗图生成器

- [ ] 渲染管线
- [ ] 内置模板
- [ ] `!meme` 指令

---

## Phase 4 · 赛博木鱼

### 数据库设计
- [ ] merit 表
- [ ] titles 表
- [ ] drops 表

### 指令实现
- [ ] `!木鱼` / `!muyu`
- [ ] `!功德` / `!merit`
- [ ] `!功德榜` / `!rank`
- [ ] `!称号` / `!title`

---

## Phase 5 · 运维完善

- [x] `!bot ping`
- [ ] `!bot stats`
- [ ] Dockerfile
- [ ] CI/CD

---

## 优先级

| 优先级 | 内容 | 状态 |
|--------|------|------|
| P0 | 命令系统 + Admin | 进行中 |
| P1 | Persona 人设模块 | 待开始 |
| P2 | 赛博木鱼 + 梗图 | 待开始 |
| P3 | 监控 + CI/CD | 待开始 |

---

## 最近完成

| 日期 | 内容 |
|------|------|
| 2026-03-05 | 实现 Persona 人设模块 (set/list/off/info 命令) |
| 2026-03-05 | 集成 SQLite 数据库和迁移系统 |
| 2026-03-05 | 添加 4 个内置人设 (毒舌程序员/赛博禅师/维基百科娘/猫娘助手) |
| 2026-03-05 | 实现 !bot avatar 命令修改 Bot 头像 |
| 2026-03-05 | 实现 !bot join 命令加入指定房间 |
| 2026-03-05 | 实现 !bot rooms 命令列出已加入房间 |
| 2026-03-05 | 实现 !bot prefix 命令热更新命令前缀 |
| 2026-03-05 | 添加 db_path 配置项 |
| 2026-03-05 | 实现 !bot name 命令修改 Bot 显示名称 |
| 2026-03-05 | 命令界面 Element 兼容卡片风格 |
| 2026-03-05 | 命令界面毛玻璃风格模板系统 |
| 2026-03-05 | Admin 命令使用毛玻璃风格模板 |
| 2026-03-05 | 命令系统基础结构 |
| 2026-03-05 | 集成命令系统到 Bot |
| 2026-03-05 | Admin 模块 (bot info/leave/ping) |