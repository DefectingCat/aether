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

### 0.2 配置扩展
- [x] 添加 `bot_owners` 配置项
- [ ] 添加 `db_path` 配置项

### 0.3 数据库层
- [ ] SQLite 集成
- [ ] migrations 目录

### 0.4 错误处理
- [ ] 自定义错误类型
- [ ] 自动重连机制

---

## Phase 1 · Admin 模块

### 指令实现
- [ ] `!bot name <名称>`
- [ ] `!bot avatar <url>`
- [x] `!bot info`
- [ ] `!bot join <room_id>`
- [x] `!bot leave`
- [ ] `!bot rooms`
- [ ] `!bot prefix <新前缀>`
- [x] `!ping`

---

## Phase 2 · Persona 人设模块

### 数据库设计
- [ ] personas 表
- [ ] room_persona 表
- [ ] chat_history 表

### 功能实现
- [ ] `!persona set <id>`
- [ ] `!persona create <id>`
- [ ] `!persona list`
- [ ] `!persona off`

### 内置人设
- [ ] 毒舌程序员
- [ ] 赛博禅师
- [ ] 维基百科娘
- [ ] 猫娘助手

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
| 2026-03-05 | 命令系统基础结构 |
| 2026-03-05 | 集成命令系统到 Bot |
| 2026-03-05 | Admin 模块 (bot info/leave/ping) |