# Element Matrix Bot — Rust 设计方案 & TODO

## 技术选型

| 层级        | Crate                            | 说明                              |
| ----------- | -------------------------------- | --------------------------------- |
| Matrix 协议 | `matrix-sdk 0.7`                 | 官方 Rust SDK，ruma 内核          |
| 异步运行时  | `tokio` (full features)          | 标准异步生态                      |
| 数据库      | `sqlx` + SQLite                  | 轻量持久化，compile-time 查询校验 |
| AI 接入     | `reqwest` + `eventsource-stream` | Anthropic SSE 流式响应            |
| 图像处理    | `image` + `imageproc`            | 梗图合成                          |
| 序列化      | `serde` + `serde_json`           | 全局数据结构                      |
| 配置        | `config` + `toml`                | 多环境配置                        |
| 错误处理    | `anyhow` + `thiserror`           | 业务错误 vs 底层错误分层          |
| 日志        | `tracing` + `tracing-subscriber` | 结构化日志                        |
| 并发状态    | `dashmap`                        | 无锁并发 HashMap                  |
| 测试        | `mockall` + `tokio-test`         | 异步 mock                         |

---

## 架构总览

```
┌──────────────────────────────────────────────────────┐
│                    Bot Entry (main.rs)                │
│         登录 → Session持久化 → EventLoop启动          │
└───────────────────────┬──────────────────────────────┘
                        │ SyncResponse Events
                        ▼
┌──────────────────────────────────────────────────────┐
│              EventDispatcher                         │
│   RoomMessage → 消息类型判断 → CommandGateway         │
│                           → MentionHandler (@bot)    │
│                           → ReactionHandler (emoji)  │
└───────────────────────┬──────────────────────────────┘
                        │
                        ▼
┌──────────────────────────────────────────────────────┐
│              CommandGateway (核心路由)                │
│                                                      │
│  前缀解析  →  权限校验  →  参数解析  →  Handler分发   │
│  "!cmd sub args..."      Permission    ArgParser     │
└──┬──────────┬────────────┬────────────┬──────────────┘
   │          │            │            │
   ▼          ▼            ▼            ▼
┌──────┐  ┌────────┐  ┌─────────┐  ┌──────────┐
│Admin │  │Persona │  │  Meme   │  │ Mokugyo  │
│Module│  │Module  │  │  Module │  │  Module  │
└──────┘  └────────┘  └─────────┘  └──────────┘
   │          │            │            │
   └──────────┴────────────┴────────────┘
                        │
                        ▼
              ┌──────────────────┐
              │   StateStore     │
              │ SQLite via sqlx  │
              └──────────────────┘
```

---

## Phase 0 · 基础框架

### 0.1 项目结构

```
aether/
├── Cargo.toml
├── .env                          # 环境变量配置
├── store/                        # Matrix session 持久化目录
└── src/
    ├── lib.rs                    # 库入口，模块导出
    ├── main.rs                   # 程序入口
    ├── config.rs                 # 配置管理（环境变量加载）
    ├── bot.rs                    # Bot 生命周期管理（登录、同步）
    ├── event_handler.rs          # 事件处理（消息、邀请）
    ├── ai_service.rs             # AI 服务封装（OpenAI API，流式响应）
    ├── conversation.rs           # 会话历史管理
    └── traits.rs                 # Trait 抽象（支持 mock 测试）
```

**当前已实现功能**:

| 功能              | 状态 | 说明                                    |
| ----------------- | ---- | --------------------------------------- |
| Matrix 客户端连接 | ✅   | 支持 session 持久化，避免重复登录       |
| 邀请处理          | ✅   | 自动接受房间邀请                        |
| 消息处理          | ✅   | 支持命令前缀、@提及、MSC3456 mentions   |
| AI 对话           | ✅   | 支持 OpenAI 兼容 API，普通/流式两种模式 |
| 会话历史          | ✅   | 内存存储，滑动窗口限制，支持重置        |
| 代理支持          | ✅   | 通过 PROXY 环境变量配置                 |
| 单元测试          | ✅   | mockall + wiremock                      |

**待实现架构扩展**:

```
src/
├── command/                      # 命令系统（待实现）
│   ├── mod.rs
│   ├── gateway.rs               # CommandGateway 路由核心
│   ├── context.rs               # CommandContext
│   ├── parser.rs                # 前缀/参数解析
│   ├── permission.rs            # 权限模型
│   └── registry.rs              # Handler 注册表
├── modules/                     # 功能模块（待实现）
│   ├── admin/
│   ├── persona/
│   │   ├── ai_client.rs
│   │   └── presets.rs
│   ├── meme/
│   │   └── renderer.rs
│   └── mokugyo/
│       ├── merit.rs
│       ├── drop.rs
│       └── title.rs
└── store/                       # 数据持久化（待实现）
    ├── mod.rs
    ├── persona_store.rs
    └── mokugyo_store.rs
```

### 0.2 核心 Trait 设计

```rust
// command/mod.rs

#[async_trait]
pub trait CommandHandler: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn subcommands(&self) -> &[SubCommand];
    fn permission(&self) -> Permission;
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()>;
}

pub struct CommandContext<'a> {
    pub client:   &'a Client,
    pub room:     Joined,
    pub sender:   OwnedUserId,
    pub cmd:      &'a str,         // 主指令
    pub sub:      Option<&'a str>, // 子指令
    pub args:     Vec<&'a str>,    // 剩余参数
    pub raw_msg:  &'a str,         // 原始消息（人设/梗图需要）
    pub event_id: OwnedEventId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Anyone,     // 任何房间成员
    RoomMod,    // power_level >= 50
    BotOwner,   // config 中指定的 owner MXID
}
```

### 0.3 TODO — 基础框架

- [ ] `Cargo.toml` 依赖配置，feature flag 分层（`full` / `minimal`）
- [ ] `AppConfig` 结构体
  - [ ] `homeserver_url`, `bot_user`, `bot_password`
  - [ ] `anthropic_api_key`, `anthropic_model`
  - [ ] `bot_owners: Vec<UserId>`
  - [ ] `command_prefix`（默认 `!`，支持运行时热更新）
  - [ ] `db_path`
- [ ] `bot.rs`：登录 + session 序列化到磁盘（避免重复设备注册）
- [ ] `CommandGateway::dispatch()`
  - [ ] 消息前缀过滤
  - [ ] `!help` 内置，遍历注册表自动生成帮助文档
  - [ ] 未知指令友好提示
- [ ] `permission.rs`：从 room state 异步查询 power_level
- [ ] `parser.rs`：`parse_command(msg) -> Option<ParsedCommand>`
  - [ ] 支持引号包裹参数：`!meme top "上方文字" "下方文字"`
- [ ] SQLite migrations，`sqlx::migrate!()`
- [ ] `tracing` 结构化日志
- [ ] 全局 panic handler + 自动重连（指数退避）

---

## Phase 1 · Admin 模块

### 指令一览

| 指令          | 参数            | 权限     | 说明                   |
| ------------- | --------------- | -------- | ---------------------- |
| `!bot name`   | `<新名称>`      | BotOwner | 修改 Bot 全局显示名    |
| `!bot avatar` | `<URL>` 或 附图 | BotOwner | 修改 Bot 头像          |
| `!bot status` | `<消息>`        | BotOwner | 修改 presence 状态消息 |
| `!bot info`   | —               | Anyone   | 查看当前 Bot 基本信息  |
| `!bot join`   | `<room_id>`     | BotOwner | 加入指定房间           |
| `!bot leave`  | —               | RoomMod  | 离开当前房间           |
| `!bot rooms`  | —               | BotOwner | 列出所有已加入房间     |
| `!bot prefix` | `<新前缀>`      | BotOwner | 修改指令前缀（热更新） |
| `!bot reload` | —               | BotOwner | 重载 config.toml       |

### TODO — Admin 模块

- [ ] `!bot name <名称>`：`account().set_display_name()` + 回复确认
- [ ] `!bot avatar <url>`
  - [ ] `reqwest` 下载 → 校验 MIME（jpeg/png/gif/webp）
  - [ ] `media().upload()` → 获取 MXC URI → `set_avatar_url()`
- [ ] `!bot avatar`（消息含 `m.image`）：从 event 直接提取 MXC，跳过下载
- [ ] `!bot info`：HTML rich text，显示显示名 / User ID / 头像 URL / 已加入房间数 / 运行时长
- [ ] `!bot leave`：发告别消息 → `room.leave()`
- [ ] 统一错误回复：`⛔ 权限不足：需要 {required_level}`
- [ ] 写操作统一 reaction 反馈：🔄 执行中 → ✅ 成功 / ❌ 失败

---

## Phase 2 · Persona 人设模块

### 指令一览

| 指令              | 参数   | 权限    | 说明                   |
| ----------------- | ------ | ------- | ---------------------- |
| `!persona set`    | `<id>` | RoomMod | 切换当前房间人设       |
| `!persona create` | `<id>` | RoomMod | 创建自定义人设         |
| `!persona edit`   | `<id>` | RoomMod | 编辑人设 system prompt |
| `!persona list`   | —      | Anyone  | 列出所有人设           |
| `!persona info`   | `<id>` | Anyone  | 查看人设详情           |
| `!persona delete` | `<id>` | RoomMod | 删除自定义人设         |
| `!persona off`    | —      | RoomMod | 关闭当前房间 AI        |
| `@bot <消息>`     | —      | Anyone  | 向当前人设发起对话     |
| `!chat clear`     | —      | Anyone  | 清除当前房间对话历史   |

### 预设人设

```rust
// modules/persona/presets.rs

pub fn builtin_personas() -> Vec<Persona> {
    vec![
        Persona {
            id: "sarcastic-dev",
            name: "毒舌程序员",
            avatar_emoji: "💻",
            system_prompt: "你是一个有20年经验的老程序员。\
                你对低质量代码感到愤怒，对 JavaScript 有刻骨的仇恨。\
                你的回答总是先吐槽，再给出正确答案。\
                你喜欢引用 Stack Overflow 嘲笑不看文档的人。\
                用中文回答，偶尔夹杂英文术语。",
        },
        Persona {
            id: "cyber-zen",
            name: "赛博禅师",
            avatar_emoji: "☯️",
            system_prompt: "你是赛博禅师，用 TCP/IP 诠释佛法，用 Git 比喻轮回。\
                说话简短而深邃，每条回复不超过100字，结尾加禅意句子。",
        },
        Persona {
            id: "wiki-chan",
            name: "维基百科娘",
            avatar_emoji: "📚",
            system_prompt: "你是维基百科的拟人，知识渊博、严谨客观。\
                回答时给出来源方向，用 [来源需引用] 标注不确定内容，语气正式，结构清晰。",
        },
        Persona {
            id: "neko-chan",
            name: "猫娘助手",
            avatar_emoji: "🐱",
            system_prompt: "你是猫娘 Neko，语气活泼可爱，句末加「喵~」。\
                乐于助人，但有时会突然分心去追激光笔。用中文回答。",
        },
    ]
}
```

### TODO — Persona 模块

- [ ] **数据库表**
  ```sql
  CREATE TABLE personas (
      id TEXT PRIMARY KEY, name TEXT NOT NULL,
      system_prompt TEXT NOT NULL, is_builtin INTEGER DEFAULT 0,
      created_by TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
  CREATE TABLE room_persona (
      room_id TEXT PRIMARY KEY, persona_id TEXT, enabled INTEGER DEFAULT 1
  );
  CREATE TABLE chat_history (
      id INTEGER PRIMARY KEY AUTOINCREMENT, room_id TEXT NOT NULL,
      role TEXT NOT NULL, content TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
  ```
- [ ] `PersonaStore`：CRUD，启动时 upsert 内置人设
- [ ] `AnthropicClient::chat()`：POST `/v1/messages`，SSE 流式读取
  - [ ] 超时 30s，指数退避重试（最多3次）
  - [ ] token 消耗日志
- [ ] Per-room `ChatSession`（`DashMap<RoomId, ChatSession>`）
  - [ ] 滑动窗口上下文（可配置最大 N 条历史）
- [ ] Mention 检测：`@bot_localpart` 或 `@full_mxid`
- [ ] `!persona create` 状态机
  - [ ] `DashMap<UserId, PendingState>` 等待下一条消息作为 prompt
  - [ ] 60s 超时自动取消
- [ ] 人设切换通知（带 emoji 和名称）

---

## Phase 3 · 梗图生成器

### 指令一览

| 指令                           | 参数   | 说明           |
| ------------------------------ | ------ | -------------- |
| `!meme list`                   | —      | 列出所有模板   |
| `!meme <模板> <文字1> [文字2]` | —      | 生成梗图       |
| `!meme add <名称>`             | + 附图 | 添加自定义模板 |
| `!meme delete <名称>`          | —      | 删除自定义模板 |

### 渲染管线

```
模板图 (PNG/JPG)  +  文字参数
         │
         ▼
┌─────────────────────────────┐
│  MemeRenderer               │
│  1. 加载模板 (image crate)   │
│  2. 加载字体 (fontdue)       │
│  3. 自动换行计算              │
│  4. 描边文字（白底黑字）       │
│  5. 编码 PNG bytes           │
└─────────────┬───────────────┘
              ▼
    matrix media upload → m.image 消息
```

### 模板配置（TOML）

```toml
# assets/meme_templates/drake.toml
id = "drake"
name = "Drake 不/是"
image = "drake.png"

[[text_regions]]
label = "不要"; x = 270; y = 10; width = 230; height = 220; font_size = 32

[[text_regions]]
label = "要"; x = 270; y = 240; width = 230; height = 220; font_size = 32
```

### TODO — 梗图模块

- [ ] 内置5个模板：Drake、两个按钮、这是X的节奏、蜘蛛侠互指、悲报/喜报
- [ ] `MemeRenderer::render(template, &[&str]) -> Result<Vec<u8>>`
  - [ ] `imageproc::draw_text_mut()` 文字渲染
  - [ ] 描边效果（4方向白色 → 中心黑色）
  - [ ] 字体内嵌 `include_bytes!`（思源黑体子集）
  - [ ] 按 width 像素自动换行
- [ ] 模板启动时加载缓存（`once_cell::sync::Lazy`）
- [ ] `!meme add`：接收图片 → 引导配置区域（交互式状态机）
- [ ] 自定义模板持久化（SQLite 存路径，文件落磁盘）
- [ ] 生成图加水印（小字 Bot 名称）

---

## Phase 4 · 赛博木鱼

### 指令一览

| 指令                | 说明               |
| ------------------- | ------------------ |
| `!木鱼` / `!muyu`   | 敲一次木鱼，+功德  |
| `!功德` / `!merit`  | 查看自己功德值     |
| `!功德榜` / `!rank` | 房间 TOP 10 排行榜 |
| `!称号` / `!title`  | 查看已解锁称号     |
| `!称号 <名称>`      | 装备/切换称号      |
| `!背包` / `!bag`    | 查看随机掉落背包   |

### 数据库设计

```sql
CREATE TABLE merit (
    user_id TEXT NOT NULL, room_id TEXT NOT NULL,
    merit_total INTEGER DEFAULT 0, merit_today INTEGER DEFAULT 0,
    last_hit DATETIME, combo INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, room_id)
);
CREATE TABLE titles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL, description TEXT, icon TEXT,
    condition TEXT NOT NULL,  -- JSON 条件
    rarity TEXT NOT NULL      -- common/rare/epic/legendary
);
CREATE TABLE user_titles (
    user_id TEXT NOT NULL, room_id TEXT NOT NULL, title_id INTEGER NOT NULL,
    obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP, equipped INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, room_id, title_id)
);
CREATE TABLE drops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL, room_id TEXT NOT NULL,
    item_name TEXT NOT NULL, item_icon TEXT, rarity TEXT NOT NULL,
    obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 功德计算设计

```rust
// modules/mokugyo/merit.rs

pub struct HitResult {
    pub merit_gained: u32,
    pub total_merit:  u64,
    pub combo:        u32,
    pub drop_result:  Option<DropItem>,
    pub new_titles:   Vec<Title>,
    pub anim:         WoodFishAnim,
}

pub enum WoodFishAnim {
    Normal,         // 🪘 *knock* +1 功德
    Combo(u32),     // 🪘💥 COMBO x{n}，加成倍率
    Critical,       // 🪘✨ 会心一击 +100 功德（1% 概率）
}
```

### 动画输出示例（Matrix HTML）

```
正常：
  🪘  *knock*
  +1 功德  |  累计：1,234 功德

连击（x7）：
  🪘💥  COMBO x7
  +10 功德（连击加成 ×1.5）

会心一击：
  🪘✨🌟✨  会心一击！
  +100 功德  |  佛光普照！

掉落：
  🎁 [史诗] 赛博念珠 📿 收入背包！
```

### 随机掉落表

```rust
// modules/mokugyo/drop.rs — 加权随机（alias method）

pub fn default_drop_table() -> DropTable {
    DropTable::new(vec![
        // Common（权重大 = 常见）
        entry("木质念珠", "📿", Common, 500),
        entry("功德符",   "📜", Common, 400),
        entry("素斋便当", "🍱", Common, 300),
        // Rare
        entry("赛博法器", "⚡", Rare, 80),
        entry("量子舍利", "💎", Rare, 60),
        // Epic
        entry("硅基菩提子", "🔮", Epic, 15),
        entry("NFT 佛像",   "🖼️", Epic, 10),
        // Legendary
        entry("赛博如来神掌", "🌟", Legendary, 2),
        entry("无限功德石",   "💫", Legendary, 1),
    ])
}
// 触发率：每次敲击 10%；会心一击时 50%
```

### 称号系统

| 称号       | 图标 | 条件             | 稀有度    |
| ---------- | ---- | ---------------- | --------- |
| 初心者     | 🌱   | 首次敲击         | Common    |
| 虔诚信徒   | 🙏   | 累计 100 功德    | Common    |
| 木鱼狂魔   | 🥁   | 单日敲击 50 次   | Rare      |
| 连击大师   | 💥   | 达成 20 连击     | Rare      |
| 会心一击者 | ⚡   | 触发 10 次会心   | Rare      |
| 赛博和尚   | 🧘   | 累计 1000 功德   | Epic      |
| 功德满满   | ✨   | 累计 10000 功德  | Epic      |
| 赛博罗汉   | 👁️   | 连续 7 天敲击    | Epic      |
| 佛祖转世   | 🌟   | 累计 100000 功德 | Legendary |

### TODO — 赛博木鱼

- [ ] **数据库层** `mokugyo_store.rs`
  - [ ] `get_merit(user_id, room_id)`
  - [ ] `add_merit(user_id, room_id, amount)` 原子更新
  - [ ] `get_leaderboard(room_id, limit)` → 带称号 JOIN 查询
  - [ ] `reset_daily_merit()` 定时任务（tokio interval，每日凌晨）
- [ ] **功德计算** `merit.rs`
  - [ ] 基础 +1 功德/次
  - [ ] 连击判定：60s 内连续算连击
  - [ ] combo >= 5 → ×1.5；combo >= 20 → ×3
  - [ ] 会心一击：1% 概率 +100 功德
  - [ ] 防刷限流：同用户 0.5s 内重复忽略（`DashMap<UserId, Instant>`）
- [ ] **掉落** `drop.rs`：alias method 加权随机，入库记录
- [ ] **称号** `title.rs`
  - [ ] 启动时初始化称号数据（upsert）
  - [ ] `check_title_unlock(ctx)` → 新解锁列表
  - [ ] `!称号` 展示（稀有度颜色：Common=灰, Rare=蓝, Epic=紫, Legendary=金 HTML span）
  - [ ] `!称号 <名称>` 切换装备
- [ ] **功德榜** `!功德榜`
  - [ ] 格式化 TOP10（带称号、功德、排名 Δ 变化 🔺🔻）
- [ ] **定时任务**
  - [ ] 每日凌晨重置 `merit_today`
  - [ ] 每周日推送本周功德总结到各房间

---

## Phase 5 · 运维完善

- [ ] `!bot ping`：响应延迟检测
- [ ] `!bot stats`：指令调用次数、AI token 消耗、DB 大小
- [ ] `SIGTERM` 优雅退出（等待进行中请求）
- [ ] 异步任务错误不 panic，`tracing::error!` 记录后继续
- [ ] `Dockerfile`（multi-stage build，final image < 50MB）
- [ ] `docker-compose.yml`（Bot + 可选 PostgreSQL 切换）
- [ ] GitHub Actions：`cargo test` + `cargo clippy` + `cargo build --release`

### 测试策略

- [ ] `parser.rs` 单元测试（空参数、引号嵌套、特殊字符）
- [ ] `permission.rs` 单元测试（各权限级别判断）
- [ ] `merit.rs` 单元测试（连击计算、会心一击概率分布验证）
- [ ] `drop.rs` 单元测试（alias method 权重分布统计）
- [ ] `renderer.rs` 集成测试（渲染输出 PNG，肉眼 review）
- [ ] E2E：测试账号在测试房间跑完整指令流程

---

## 开发节奏建议

```
Week 1  Phase 0 框架 + Phase 1 Admin     跑通主流程
Week 2  Phase 2 Persona                  AI 对话核心
Week 3  Phase 4 木鱼                     数据库 + 游戏逻辑
Week 4  Phase 3 梗图                     图像处理
Week 5  Phase 5 完善 + 测试 + 部署
```

> **原则**：每个 Phase 完成后必须能独立演示；所有模块通过 feature flag 控制编译，`--features=all` 启用全部；未完成模块不阻塞主流程。
