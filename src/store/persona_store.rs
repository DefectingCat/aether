//! Persona 人设存储

use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 人设定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// 人设 ID
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 头像 emoji
    pub avatar_emoji: Option<String>,
    /// 是否内置
    pub is_builtin: bool,
    /// 创建者
    pub created_by: Option<String>,
}

/// 人设存储
#[derive(Clone)]
pub struct PersonaStore {
    conn: Arc<Mutex<Connection>>,
}

impl PersonaStore {
    /// 创建新的人设存储
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 初始化内置人设
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

    /// 获取所有人设
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

    /// 根据 ID 获取人设
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

    /// 设置房间人设
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

    /// 获取房间当前人设
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

    /// 关闭房间人设
    pub fn disable_room_persona(&self, room_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE room_persona SET enabled = 0 WHERE room_id = ?1",
            [room_id],
        )?;

        tracing::debug!("房间 {} 已关闭人设", room_id);
        Ok(())
    }

    /// 创建自定义人设
    pub fn create_persona(&self, persona: &Persona) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO personas (id, name, system_prompt, avatar_emoji, is_builtin, created_by) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![persona.id, persona.name, persona.system_prompt, persona.avatar_emoji, persona.created_by],
        )?;

        tracing::debug!("创建人设: {}", persona.id);
        Ok(())
    }

    /// 删除自定义人设
    pub fn delete_persona(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM personas WHERE id = ?1 AND is_builtin = 0",
            [id],
        )?;

        Ok(rows_affected > 0)
    }
}
