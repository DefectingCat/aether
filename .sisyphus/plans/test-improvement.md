# Aether Matrix 项目测试完善方案

## TL;DR

> **目标**: 建立完整的测试覆盖体系，从当前 ~60% 覆盖率提升到 90%+，确保核心功能和边缘情况都有充分验证。
>
> **策略**: 采用分层测试方法 - 单元测试（函数级） + 集成测试（组件级） + 端到端测试（系统级）
>
> **交付**: 45+ 个新测试用例，修复现有编译错误，建立统一的测试基础设施

---

## 工作目标

### 核心目标
1. **修复现有问题**：解决编译错误，统一测试策略
2. **全面覆盖**：为所有关键模块添加缺失的测试
3. **质量保证**：包含错误处理、边界情况、并发场景
4. **可持续性**：建立清晰的测试模式，便于未来维护

### 具体可交付成果
- [x] 修复所有编译错误
- [x] 为 `store/` 添加 15+ 个数据库测试
- [x] 为 `modules/` 添加 20+ 个命令处理器测试  
- [ ] 为 `mcp/` 添加 10+ 个 MCP 功能测试
- [ ] 为现有模块添加 20+ 个错误路径和边界情况测试
- [x] 统一 Mock 策略，创建共享测试工具
- [ ] 建立测试覆盖率报告

---

## Wave 1: 基础修复和存储层测试

### Task 1: 修复编译错误和统一测试框架
- [x] 修复 `mcp_integration.rs` 中的 `AiShip` → `AiService` 错误
- [x] 创建统一的测试工具模块 `tests/common/test_utils.rs`
- [x] 统一使用 `mockall` 作为主要 Mock 策略
- [x] 为所有测试添加适当的日志级别配置

**Category**: `quick`
**Skills**: [`git-master`, `rust-symbol-analyzer`]
**Acceptance**: `cargo test` 编译通过，所有现有测试通过

### Task 2: 数据库连接和迁移测试
- [x] 创建 `DatabaseConnectionTest` 模块
- [x] 测试数据库连接建立
- [x] 测试表结构创建和迁移
- [x] 测试连接池和并发访问

**Category**: `deep`
**Skills**: [`m13-domain-error`, `rust-symbol-analyzer`]
**Acceptance**: 测试文件创建，`cargo test database` → PASS

### Task 3: PersonaStore CRUD 操作测试
- [x] 测试内置人设初始化
- [x] 测试自定义人设创建、读取、更新、删除
- [x] 测试房间绑定操作
- [x] 测试错误处理（重复 ID、无效数据等）

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，10+ test cases

### Task 4: 聊天历史存储测试
- [x] 确认 chat_history 功能是否存在
- [x] 发现：表存在但实现缺失（已记录到 issues.md）
- [x] 无法创建测试（功能不存在）

**Category**: `deep`
**Skills**: [`m07-concurrency`, `m13-domain-error`]
**Acceptance**: 测试文件创建，8+ test cases

---

## Wave 2: 命令系统测试

### Task 5: Admin 模块命令处理器测试
- [x] 测试 `!bot info`、`!bot ping`、`!leave` 等命令
- [x] 测试权限检查（BotOwner vs RoomMod vs Anyone）
- [x] 测试参数验证和错误处理
- [x] 测试矩阵客户端交互

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，12+ test cases

### Task 6: Persona 模块命令处理器测试
- [x] 测试 `!persona list`、`!persona set`、`!persona create` 等命令
- [x] 测试人设绑定到房间的功能
- [x] 测试自定义人设的创建和删除
- [x] 测试内置人设的保护机制

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，15+ test cases

### Task 7: MCP 模块命令处理器测试
- [x] 测试 `!mcp list`、`!mcp servers`、`!mcp reload` 命令
- [x] 测试 MCP 工具的展示和管理
- [x] 测试服务器状态查询
- [x] 测试配置重载功能

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，8+ test cases

### Task 8: 赛博木鱼模块测试
- [x] 测试 `!木鱼`、`!功德`、`!功德榜`、`!称号`、`!背包` 命令
- [x] 测试功德计算和存储
- [x] 测试排行榜功能
- [x] 测试物品和称号系统

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，10+ test cases

### Task 9: 命令权限验证测试
- [x] 测试三级权限模型（Anyone/RoomMod/BotOwner）
- [x] 测试私聊房间的特殊权限处理
- [x] 测试权限检查的边界情况
- [x] 测试权限错误的用户反馈

**Category**: `deep`
**Skills**: [`m09-domain`, `m13-domain-error`]
**Acceptance**: 测试文件创建，12+ test cases

---

## Wave 3: MCP 功能测试

### Task 10: 内置工具执行测试
- [x] 测试 WebFetch 工具的 URL 获取功能
- [x] 测试内容长度限制
- [x] 测试超时处理
- [x] 测试错误 URL 处理

**Category**: `deep`
**Skills**: [`m13-domain-error`, `domain-web`]
**Acceptance**: 测试文件创建，8+ test cases

### Task 11: 外部 MCP 服务器管理测试
- [x] 测试外部 MCP 服务器的启动和停止
- [x] 测试服务器连接状态管理
- [x] 测试服务器配置加载
- [x] 测试服务器错误恢复

**Category**: `deep`
**Skills**: [`m07-concurrency`, `m13-domain-error`]
**Acceptance**: 测试文件创建，6+ test cases

### Task 12: 工具注册表和转换测试
- [x] 测试工具注册和发现
- [x] 测试 OpenAI 工具格式转换
- [x] 测试工具参数验证
- [x] 测试工具执行委托

**Category**: `deep`
**Skills**: [`m05-type-driven`, `m13-domain-error`]
**Acceptance**: 测试文件创建，8+ test cases

### Task 13: MCP 配置加载测试
- [x] 测试环境变量配置解析
- [x] 测试 TOML 配置文件加载
- [x] 测试配置验证和默认值
- [x] 测试配置合并逻辑

**Category**: `deep`
**Skills**: [`m13-domain-error`, `coding-guidelines`]
**Acceptance**: 测试文件创建，6+ test cases

### Task 14: 工具执行重试机制测试
- [x] 测试工具执行失败时的重试逻辑
- [x] 测试重试次数限制
- [x] 测试退避延迟
- [x] 测试最终失败处理

**Category**: `deep`
**Skills**: [`m13-domain-error`, `m10-performance`]
**Acceptance**: 测试文件创建，5+ test cases

---

## Wave 4: 核心服务增强测试

### Task 15: AiService 错误处理测试
- [ ] 测试 API 密钥无效时的错误处理
- [ ] 测试网络连接失败的重试
- [ ] 测试速率限制处理
- [ ] 测试模型不可用的错误

**Category**: `deep`
**Skills**: [`m13-domain-error`, `domain-web`]
**Acceptance**: 测试文件创建，8+ test cases

### Task 16: ConversationManager 边界情况测试
- [x] 测试空消息处理
- [x] 测试超长消息截断
- [x] 测试极端历史长度设置
- [x] 测试并发会话操作

**Category**: `deep`
**Skills**: [`m07-concurrency`, `m13-domain-error`]
**Acceptance**: 添加到现有文件，6+ new test cases

### Task 17: Media 处理边界情况测试
- [x] 测试无效图片格式处理
- [x] 测试超大图片内存限制
- [x] 测试损坏图片文件处理
- [x] 测试空/无效 Data URL 处理

**Category**: `deep`
**Skills**: [`m13-domain-error`, `domain-ml`]
**Acceptance**: 添加到现有文件，5+ new test cases

### Task 18: EventHandler 错误路径测试
- [ ] 测试无效消息格式处理
- [ ] 测试未知命令处理
- [ ] 测试消息发送失败处理
- [ ] 测试事件处理并发安全

**Category**: `deep`
**Skills**: [`m13-domain-error`, `m07-concurrency`]
**Acceptance**: 测试文件创建，6+ test cases

### Task 19: Config 解析和验证测试
- [ ] 测试环境变量缺失处理
- [ ] 测试无效配置值验证
- [ ] 测试配置默认值应用
- [ ] 测试敏感信息保护

**Category**: `deep`
**Skills**: [`m13-domain-error`, `coding-guidelines`]
**Acceptance**: 测试文件创建，8+ test cases

---

## Wave 5: 测试基础设施

### Task 20: 共享 Mock 工具统一
- [x] 统一使用 `mockall` 作为主要 Mock 策略
- [x] 创建共享的 Mock 工具模块
- [x] 迁移现有手动 Mock 到 `mockall`
- [x] 更新测试文档

**Category**: `quick`
**Skills**: [`git-master`, `rust-refactor-helper`]
**Acceptance**: Shared mock utilities created

### Task 21: 测试覆盖率配置
- [x] 集成 `cargo-tarpaulin` 进行覆盖率分析
- [x] 配置覆盖率报告生成
- [x] 设置覆盖率阈值
- [x] 创建覆盖率 badge

**Category**: `quick`
**Skills**: [`git-master`, `coding-guidelines`]
**Acceptance**: Coverage report generated

### Task 22: 文档和最佳实践
- [ ] 创建测试编写指南
- [ ] 更新 CONTRIBUTING.md
- [ ] 添加测试模式示例
- [ ] 创建常见测试场景模板

**Category**: `writing`
**Skills**: [`coding-guidelines`]
**Acceptance**: Documentation created

### Task 23: CI/CD 集成
- [x] 配置 GitHub Actions 运行测试
- [x] 添加覆盖率检查到 PR 流程
- [x] 设置测试缓存优化
- [x] 配置测试并行执行

**Category**: `quick`
**Skills**: [`git-master`, `domain-cloud-native`]
**Acceptance**: CI workflow configured

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
- [ ] F2. **Code Quality Review** — `unspecified-high`
- [ ] F3. **Real Manual QA** — `unspecified-high`
- [ ] F4. **Scope Fidelity Check** — `deep`