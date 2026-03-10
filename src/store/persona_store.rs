//! Persona 人设存储模块。
//!
//! 提供人设（Persona）的定义、存储和管理功能。人设用于自定义 AI 助手的
//! 响应风格和系统提示词，每个 Matrix 房间可以设置独立的人设。

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 人设定义，包含 AI 助手的性格和行为配置。
///
/// 人设用于自定义 AI 的响应风格和系统提示词。
/// 每个房间可以设置独立的人设，影响该房间的对话体验。
///
/// # 内置人设
///
/// 项目预置了 4 个内置人设：
/// - `sarcastic-dev`: 毒舌程序员，20年经验，对低质量代码愤怒
/// - `cyber-zen`: 赛博禅师，用 TCP/IP 诠释佛法，简短深邃
/// - `wiki-chan`: 维基百科娘，知识渊博、严谨客观、标注来源
/// - `neko-chan`: 猫娘助手，语气活泼可爱，句末加「喵~」
///
/// # 自定义人设
///
/// 用户可以通过 `!persona create` 命令创建自定义人设，
/// 自定义人设存储在数据库中，可以被删除。
///
/// # Example
///
/// ```no_run
/// use aether_matrix::store::Persona;
///
/// // 创建自定义人设
/// let persona = Persona {
///     id: "my-assistant".to_string(),
///     name: "我的助手".to_string(),
///     system_prompt: "你是一个专业的 Rust 开发顾问...".to_string(),
///     avatar_emoji: Some("🦀".to_string()),
///     is_builtin: false,
///     created_by: Some("@user:matrix.org".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// 人设唯一标识符。
    ///
    /// 用于在命令中引用人设，如 `!persona set sarcastic-dev`。
    /// 内置人设使用连字符分隔的小写字母，如 `sarcastic-dev`。
    /// 自定义人设建议使用相同的命名风格。
    ///
    /// # Example
    ///
    /// - `sarcastic-dev` - 内置人设
    /// - `my-custom-assistant` - 自定义人设
    pub id: String,

    /// 人设显示名称。
    ///
    /// 在人设列表和状态中显示的友好名称。
    ///
    /// # Example
    ///
    /// - `"毒舌程序员"`
    /// - `"赛博禅师"`
    /// - `"我的助手"`
    pub name: String,

    /// 系统提示词。
    ///
    /// 发送给 AI 模型的系统提示，定义 AI 的行为和响应风格。
    /// 应该详细描述人设的性格、语气和专业领域。
    ///
    /// # Example
    ///
    /// ```ignore
    /// "你是一个有20年经验的老程序员。你对低质量代码感到愤怒..."
    /// ```
    pub system_prompt: String,

    /// 头像 Emoji（可选）。
    ///
    /// 用于在人设列表中显示的可选表情符号。
    ///
    /// # Example
    ///
    /// - `Some("💻".to_string())` - 毒舌程序员
    /// - `Some("🐱".to_string())` - 猫娘助手
    /// - `None` - 无头像
    pub avatar_emoji: Option<String>,

    /// 是否为内置人设。
    ///
    /// 内置人设由系统预置，不可删除。
    /// 自定义人设由用户创建，可以删除。
    pub is_builtin: bool,

    /// 创建者用户 ID（可选）。
    ///
    /// 对于内置人设，此值为 `None`。
    /// 对于自定义人设，记录创建者的 Matrix 用户 ID。
    ///
    /// # Example
    ///
    /// - `None` - 内置人设
    /// - `Some("@user:matrix.org".to_string())` - 自定义人设
    pub created_by: Option<String>,
}

/// 人设存储，管理内置和自定义人设的持久化。
///
/// 提供人设的 CRUD 操作，以及房间与人设的关联管理。
/// 内部使用 SQLite 数据库存储，通过 `Arc<Mutex<Connection>>` 实现线程安全。
///
/// # 内置人设
///
/// 系统预置 4 个内置人设，通过 [`PersonaStore::init_builtin_personas`] 初始化。
/// 内置人设不可删除，但可以通过 `!persona set` 命令应用到房间。
///
/// # 房间人设
///
/// 每个房间可以设置一个激活的人设：
/// - [`PersonaStore::set_room_persona`] - 设置房间人设
/// - [`PersonaStore::get_room_persona`] - 获取房间当前人设
/// - [`PersonaStore::disable_room_persona`] - 关闭房间人设
///
/// # Example
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use rusqlite::Connection;
/// use aether_matrix::store::PersonaStore;
///
/// // 创建数据库连接
/// let conn = Connection::open("./data/aether.db")?;
/// let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
///
/// // 初始化内置人设
/// store.init_builtin_personas()?;
///
/// // 获取所有人设
/// let personas = store.get_all()?;
/// for persona in personas {
///     println!("{}: {}", persona.id, persona.name);
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone)]
pub struct PersonaStore {
    conn: Arc<Mutex<Connection>>,
}

impl PersonaStore {
    /// 创建新的人设存储实例。
    ///
    /// # Arguments
    ///
    /// * `conn` - SQLite 数据库连接，包装在 `Arc<Mutex<>>` 中以实现线程安全
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::sync::{Arc, Mutex};
    /// use rusqlite::Connection;
    /// use aether_matrix::store::PersonaStore;
    ///
    /// let conn = Connection::open("./data/aether.db")?;
    /// let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 初始化内置人设。
    ///
    /// 将 4 个预置人设插入数据库。使用 UPSERT 语义，
    /// 如果人设已存在则更新，保证幂等性。
    ///
    /// # Errors
    ///
    /// 当数据库操作失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    ///
    /// store.init_builtin_personas()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn init_builtin_personas(&self) -> Result<()> {
        let builtin_personas = Self::builtin_personas();
        let conn = self.conn.lock().unwrap();

        for persona in builtin_personas {
            // 使用 UPSERT 插入或更新
            conn.execute(
                r#"
                INSERT INTO personas (id, name, system_prompt, avatar_emoji, is_builtin)
                VALUES (?1, ?2, ?3, ?4, 1)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    system_prompt = excluded.system_prompt,
                    avatar_emoji = excluded.avatar_emoji
                "#,
                params![
                    persona.id,
                    persona.name,
                    persona.system_prompt,
                    persona.avatar_emoji
                ],
            )?;
        }

        tracing::info!("内置人设初始化完成");
        Ok(())
    }

    /// 获取内置人设列表
    fn builtin_personas() -> Vec<Persona> {
        vec![
            Persona {
                id: "sarcastic-dev".to_string(),
                name: "毒舌程序员".to_string(),
                system_prompt: "你是一个有20年经验的老程序员。\
                    你对低质量代码感到愤怒，对 JavaScript 有刻骨的仇恨。\
                    你的回答总是先吐槽，再给出正确答案。\
                    你喜欢引用 Stack Overflow 嘲笑不看文档的人。\
                    用中文回答，偶尔夹杂英文术语。"
                    .to_string(),
                avatar_emoji: Some("💻".to_string()),
                is_builtin: true,
                created_by: None,
            },
            Persona {
                id: "cyber-zen".to_string(),
                name: "赛博禅师".to_string(),
                system_prompt: "你是赛博禅师，用 TCP/IP 诠释佛法，用 Git 比喻轮回。\
                    说话简短而深邃，每条回复不超过100字，结尾加禅意句子。"
                    .to_string(),
                avatar_emoji: Some("☯️".to_string()),
                is_builtin: true,
                created_by: None,
            },
            Persona {
                id: "wiki-chan".to_string(),
                name: "维基百科娘".to_string(),
                system_prompt: "你是维基百科的拟人，知识渊博、严谨客观。\
                    回答时给出来源方向，用 [来源需引用] 标注不确定内容，语气正式，结构清晰。"
                    .to_string(),
                avatar_emoji: Some("📚".to_string()),
                is_builtin: true,
                created_by: None,
            },
            Persona {
                id: "neko-chan".to_string(),
                name: "猫娘助手".to_string(),
                system_prompt: "你是猫娘 Neko，语气活泼可爱，句末加「喵~」。\
                    乐于助人，但有时会突然分心去追激光笔。用中文回答。"
                    .to_string(),
                avatar_emoji: Some("🐱".to_string()),
                is_builtin: true,
                created_by: None,
            },
        ]
    }

    /// 获取所有人设列表。
    ///
    /// 返回所有内置和自定义人设，按内置优先、名称升序排列。
    ///
    /// # Returns
    ///
    /// 成功时返回人设列表。如果没有数据，返回空列表。
    ///
    /// # Errors
    ///
    /// 当数据库查询失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// # store.init_builtin_personas()?;
    /// let personas = store.get_all()?;
    /// for persona in personas {
    ///     println!("{}: {} {}", persona.id, persona.avatar_emoji.unwrap_or_default(), persona.name);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_all(&self) -> Result<Vec<Persona>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, system_prompt, avatar_emoji, is_builtin, created_by FROM personas ORDER BY is_builtin DESC, name ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Persona {
                id: row.get(0)?,
                name: row.get(1)?,
                system_prompt: row.get(2)?,
                avatar_emoji: row.get(3)?,
                is_builtin: row.get::<_, i32>(4)? != 0,
                created_by: row.get(5)?,
            })
        })?;

        let mut personas = Vec::new();
        for row in rows {
            personas.push(row?);
        }

        Ok(personas)
    }

    /// 根据 ID 获取人设。
    ///
    /// 查询数据库中指定 ID 的人设记录。
    ///
    /// # Arguments
    ///
    /// * `id` - 人设唯一标识符
    ///
    /// # Returns
    ///
    /// - `Some(Persona)` - 找到匹配的人设
    /// - `None` - 未找到匹配的人设
    ///
    /// # Errors
    ///
    /// 当数据库查询失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// # store.init_builtin_personas()?;
    /// if let Some(persona) = store.get_by_id("sarcastic-dev")? {
    ///     println!("找到人设: {}", persona.name);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_by_id(&self, id: &str) -> Result<Option<Persona>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, system_prompt, avatar_emoji, is_builtin, created_by FROM personas WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map([id], |row| {
            Ok(Persona {
                id: row.get(0)?,
                name: row.get(1)?,
                system_prompt: row.get(2)?,
                avatar_emoji: row.get(3)?,
                is_builtin: row.get::<_, i32>(4)? != 0,
                created_by: row.get(5)?,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// 设置房间人设。
    ///
    /// 将指定人设绑定到房间，影响该房间的 AI 响应风格。
    /// 如果房间已有设置的人设，将更新为新人设。
    ///
    /// # Arguments
    ///
    /// * `room_id` - Matrix 房间 ID，如 `!room:matrix.org`
    /// * `persona_id` - 人设唯一标识符
    /// * `set_by` - 执行设置的用户 Matrix ID
    ///
    /// # Errors
    ///
    /// 当数据库操作失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// # store.init_builtin_personas()?;
    /// store.set_room_persona("!room:matrix.org", "sarcastic-dev", "@user:matrix.org")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn set_room_persona(&self, room_id: &str, persona_id: &str, set_by: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO room_persona (room_id, persona_id, enabled, set_by)
            VALUES (?1, ?2, 1, ?3)
            ON CONFLICT(room_id) DO UPDATE SET
                persona_id = excluded.persona_id,
                enabled = 1,
                set_by = excluded.set_by,
                set_at = CURRENT_TIMESTAMP
            "#,
            params![room_id, persona_id, set_by],
        )?;

        tracing::debug!("房间 {} 设置人设为 {}", room_id, persona_id);
        Ok(())
    }

    /// 获取房间当前激活的人设。
    ///
    /// 查询房间设置的人设，仅返回已启用的人设。
    ///
    /// # Arguments
    ///
    /// * `room_id` - Matrix 房间 ID
    ///
    /// # Returns
    ///
    /// - `Some(Persona)` - 房间已设置且启用了人设
    /// - `None` - 房间未设置人设或人设已禁用
    ///
    /// # Errors
    ///
    /// 当数据库查询失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// # store.init_builtin_personas()?;
    /// # store.set_room_persona("!room:matrix.org", "sarcastic-dev", "@user:matrix.org")?;
    /// if let Some(persona) = store.get_room_persona("!room:matrix.org")? {
    ///     println!("房间人设: {}", persona.name);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_room_persona(&self, room_id: &str) -> Result<Option<Persona>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT p.id, p.name, p.system_prompt, p.avatar_emoji, p.is_builtin, p.created_by
            FROM room_persona rp
            JOIN personas p ON rp.persona_id = p.id
            WHERE rp.room_id = ?1 AND rp.enabled = 1
            "#,
        )?;

        let mut rows = stmt.query_map([room_id], |row| {
            Ok(Persona {
                id: row.get(0)?,
                name: row.get(1)?,
                system_prompt: row.get(2)?,
                avatar_emoji: row.get(3)?,
                is_builtin: row.get::<_, i32>(4)? != 0,
                created_by: row.get(5)?,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// 关闭房间人设。
    ///
    /// 将房间的人设设置为禁用状态，不再影响 AI 响应。
    ///
    /// # Arguments
    ///
    /// * `room_id` - Matrix 房间 ID
    ///
    /// # Errors
    ///
    /// 当数据库操作失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// store.disable_room_persona("!room:matrix.org")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn disable_room_persona(&self, room_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE room_persona SET enabled = 0 WHERE room_id = ?1",
            [room_id],
        )?;

        tracing::debug!("房间 {} 已关闭人设", room_id);
        Ok(())
    }

    /// 创建自定义人设。
    ///
    /// 将新的人设插入数据库。注意 `is_builtin` 字段会被强制设为 `false`。
    ///
    /// # Arguments
    ///
    /// * `persona` - 要创建的人设数据
    ///
    /// # Errors
    ///
    /// 当以下情况发生时返回错误：
    /// - 人设 ID 已存在
    /// - 数据库操作失败
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::store::{Persona, PersonaStore};
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    ///
    /// let persona = Persona {
    ///     id: "my-assistant".to_string(),
    ///     name: "我的助手".to_string(),
    ///     system_prompt: "你是一个专业的 Rust 开发顾问...".to_string(),
    ///     avatar_emoji: Some("🦀".to_string()),
    ///     is_builtin: false,
    ///     created_by: Some("@user:matrix.org".to_string()),
    /// };
    /// store.create_persona(&persona)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn create_persona(&self, persona: &Persona) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO personas (id, name, system_prompt, avatar_emoji, is_builtin, created_by) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![persona.id, persona.name, persona.system_prompt, persona.avatar_emoji, persona.created_by],
        )?;

        tracing::debug!("创建人设: {}", persona.id);
        Ok(())
    }

    /// 删除自定义人设。
    ///
    /// 仅能删除自定义人设，内置人设不可删除。
    ///
    /// # Arguments
    ///
    /// * `id` - 要删除的人设 ID
    ///
    /// # Returns
    ///
    /// - `true` - 成功删除人设
    /// - `false` - 人设不存在或是内置人设
    ///
    /// # Errors
    ///
    /// 当数据库操作失败时返回错误。
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aether_matrix::store::PersonaStore;
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # let conn = Connection::open(":memory:")?;
    /// # let store = PersonaStore::new(Arc::new(Mutex::new(conn)));
    /// // 删除自定义人设
    /// if store.delete_persona("my-custom-persona")? {
    ///     println!("人设已删除");
    /// } else {
    ///     println!("人设不存在或是内置人设");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn delete_persona(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM personas WHERE id = ?1 AND is_builtin = 0",
            [id],
        )?;

        Ok(rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::TempDir;

    fn create_test_store() -> (PersonaStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch(include_str!("../../migrations/20260305000000_init.sql"))
            .unwrap();

        (PersonaStore::new(Arc::new(Mutex::new(conn))), temp_dir)
    }

    #[test]
    fn test_init_builtin_personas_creates_4_personas() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let personas = store.get_all().unwrap();
        assert_eq!(personas.len(), 4);
    }

    #[test]
    fn test_init_builtin_personas_are_marked_as_builtin() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let personas = store.get_all().unwrap();
        for persona in personas {
            assert!(persona.is_builtin);
        }
    }

    #[test]
    fn test_init_builtin_personas_idempotent() {
        let (store, _temp_dir) = create_test_store();

        store.init_builtin_personas().unwrap();
        let first_count = store.get_all().unwrap().len();

        store.init_builtin_personas().unwrap();
        let second_count = store.get_all().unwrap().len();

        assert_eq!(first_count, second_count);
    }

    #[test]
    fn test_builtin_personas_have_valid_data() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let personas = store.get_all().unwrap();
        for persona in personas {
            assert!(!persona.id.is_empty());
            assert!(!persona.name.is_empty());
            assert!(!persona.system_prompt.is_empty());
        }
    }

    #[test]
    fn test_get_all_returns_empty_when_no_personas() {
        let (store, _temp_dir) = create_test_store();
        let personas = store.get_all().unwrap();
        assert!(personas.is_empty());
    }

    #[test]
    fn test_get_all_returns_builtin_first() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let custom_persona = Persona {
            id: "aaa-custom".to_string(),
            name: "A Custom".to_string(),
            system_prompt: "Custom prompt".to_string(),
            avatar_emoji: None,
            is_builtin: false,
            created_by: None,
        };
        store.create_persona(&custom_persona).unwrap();

        let personas = store.get_all().unwrap();
        assert_eq!(personas.len(), 5);

        for persona in &personas[..4] {
            assert!(persona.is_builtin);
        }
        assert!(!personas[4].is_builtin);
    }

    #[test]
    fn test_get_by_id_returns_persona() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let persona = store.get_by_id("sarcastic-dev").unwrap();
        assert!(persona.is_some());
        assert_eq!(persona.unwrap().name, "毒舌程序员");
    }

    #[test]
    fn test_get_by_id_returns_none_for_nonexistent() {
        let (store, _temp_dir) = create_test_store();
        let persona = store.get_by_id("nonexistent").unwrap();
        assert!(persona.is_none());
    }

    #[test]
    fn test_create_persona_success() {
        let (store, _temp_dir) = create_test_store();

        let persona = Persona {
            id: "custom-1".to_string(),
            name: "Custom Persona".to_string(),
            system_prompt: "You are custom.".to_string(),
            avatar_emoji: Some("🎯".to_string()),
            is_builtin: false,
            created_by: Some("@user:matrix.org".to_string()),
        };

        store.create_persona(&persona).unwrap();

        let retrieved = store.get_by_id("custom-1").unwrap().unwrap();
        assert_eq!(retrieved.name, "Custom Persona");
        assert_eq!(retrieved.created_by, Some("@user:matrix.org".to_string()));
    }

    #[test]
    fn test_create_persona_sets_is_builtin_false() {
        let (store, _temp_dir) = create_test_store();

        let persona = Persona {
            id: "custom-2".to_string(),
            name: "Custom".to_string(),
            system_prompt: "Prompt".to_string(),
            avatar_emoji: None,
            is_builtin: true,
            created_by: None,
        };

        store.create_persona(&persona).unwrap();

        let retrieved = store.get_by_id("custom-2").unwrap().unwrap();
        assert!(!retrieved.is_builtin);
    }

    #[test]
    fn test_delete_persona_removes_custom_persona() {
        let (store, _temp_dir) = create_test_store();

        let persona = Persona {
            id: "to-delete".to_string(),
            name: "To Delete".to_string(),
            system_prompt: "Will be deleted".to_string(),
            avatar_emoji: None,
            is_builtin: false,
            created_by: None,
        };
        store.create_persona(&persona).unwrap();

        let deleted = store.delete_persona("to-delete").unwrap();
        assert!(deleted);

        let retrieved = store.get_by_id("to-delete").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_delete_persona_fails_for_builtin() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        let deleted = store.delete_persona("sarcastic-dev").unwrap();
        assert!(!deleted);

        let still_exists = store.get_by_id("sarcastic-dev").unwrap();
        assert!(still_exists.is_some());
    }

    #[test]
    fn test_set_room_persona_creates_association() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        store
            .set_room_persona("!room1:matrix.org", "sarcastic-dev", "@user:matrix.org")
            .unwrap();

        let persona = store.get_room_persona("!room1:matrix.org").unwrap();
        assert!(persona.is_some());
        assert_eq!(persona.unwrap().id, "sarcastic-dev");
    }

    #[test]
    fn test_get_room_persona_returns_set_persona() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        store
            .set_room_persona("!room2:matrix.org", "cyber-zen", "@user:matrix.org")
            .unwrap();

        let persona = store.get_room_persona("!room2:matrix.org").unwrap();
        assert!(persona.is_some());
        assert_eq!(persona.unwrap().id, "cyber-zen");
    }

    #[test]
    fn test_disable_room_persona_sets_enabled_false() {
        let (store, _temp_dir) = create_test_store();
        store.init_builtin_personas().unwrap();

        store
            .set_room_persona("!room3:matrix.org", "wiki-chan", "@user:matrix.org")
            .unwrap();

        let before = store.get_room_persona("!room3:matrix.org").unwrap();
        assert!(before.is_some());

        store.disable_room_persona("!room3:matrix.org").unwrap();

        let after = store.get_room_persona("!room3:matrix.org").unwrap();
        assert!(after.is_none());
    }
}
