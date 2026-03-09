//! 赛博木鱼存储层

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

use super::models::*;

/// 赛博木鱼存储
#[derive(Clone)]
pub struct MuyuStore {
    conn: Arc<Mutex<Connection>>,
}

impl MuyuStore {
    /// 创建新的存储实例
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // ==================== 功德操作 ====================

    /// 获取用户功德记录
    pub fn get_merit(&self, user_id: &str, room_id: &str) -> Result<Option<MeritRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT user_id, room_id, merit_total, merit_today, hits_today, last_hit, combo, max_combo, critical_count, consecutive_days, last_hit_date
             FROM merit WHERE user_id = ?1 AND room_id = ?2"
        )?;

        let mut rows = stmt.query_map(params![user_id, room_id], |row| {
            Ok(MeritRecord {
                user_id: row.get(0)?,
                room_id: row.get(1)?,
                merit_total: row.get(2)?,
                merit_today: row.get(3)?,
                hits_today: row.get(4)?,
                last_hit: row.get::<_, Option<String>>(5)?.and_then(|s| s.parse().ok()),
                combo: row.get(6)?,
                max_combo: row.get(7)?,
                critical_count: row.get(8)?,
                consecutive_days: row.get(9)?,
                last_hit_date: row.get::<_, Option<String>>(10)?.and_then(|s| s.parse().ok()),
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// 更新功德记录（原子操作）
    pub fn update_merit(
        &self,
        user_id: &str,
        room_id: &str,
        merit_delta: i64,
        new_combo: i32,
        is_critical: bool,
    ) -> Result<MeritRecord> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let today = now.date_naive();
        let today_str = today.to_string();

        // 检查是否是新的一天，需要重置每日计数
        let is_new_day = {
            let mut stmt = conn.prepare(
                "SELECT last_hit_date FROM merit WHERE user_id = ?1 AND room_id = ?2"
            )?;
            let last_date: Option<String> = stmt.query_row(params![user_id, room_id], |row| row.get(0)).ok();
            last_date.as_deref() != Some(&today_str)
        };

        // UPSERT 操作
        conn.execute(
            r#"
            INSERT INTO merit (user_id, room_id, merit_total, merit_today, hits_today, last_hit, combo, max_combo, critical_count, last_hit_date)
            VALUES (?1, ?2, ?3, 1, 1, ?4, ?5, ?5, ?6, ?7)
            ON CONFLICT(user_id, room_id) DO UPDATE SET
                merit_total = merit_total + ?3,
                merit_today = CASE WHEN ?8 = 1 THEN merit_today + ?3 ELSE ?3 END,
                hits_today = CASE WHEN ?8 = 1 THEN hits_today + 1 ELSE 1 END,
                last_hit = ?4,
                combo = ?5,
                max_combo = MAX(max_combo, ?5),
                critical_count = critical_count + CASE WHEN ?9 = 1 THEN 1 ELSE 0 END,
                last_hit_date = ?7,
                consecutive_days = CASE
                    WHEN ?8 = 1 THEN consecutive_days
                    ELSE consecutive_days + 1
                END
            "#,
            params![
                user_id,
                room_id,
                merit_delta,
                now_str,
                new_combo,
                if is_critical { 1 } else { 0 },
                today_str,
                if is_new_day { 0 } else { 1 },
                if is_critical { 1 } else { 0 },
            ],
        )?;

        // 返回更新后的记录
        drop(conn);
        self.get_merit(user_id, room_id)?.ok_or_else(|| anyhow::anyhow!("Failed to get merit after update"))
    }

    /// 重置连击（超过时间窗口时）
    pub fn reset_combo(&self, user_id: &str, room_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE merit SET combo = 0 WHERE user_id = ?1 AND room_id = ?2",
            params![user_id, room_id],
        )?;
        Ok(())
    }

    /// 获取排行榜
    pub fn get_leaderboard(&self, room_id: &str, limit: usize) -> Result<Vec<LeaderboardEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT m.user_id, m.merit_total,
                   t.id, t.name, t.description, t.icon, t.condition_kind, t.condition_value, t.rarity
            FROM merit m
            LEFT JOIN user_titles ut ON m.user_id = ut.user_id AND m.room_id = ut.room_id AND ut.equipped = 1
            LEFT JOIN titles t ON ut.title_id = t.id
            WHERE m.room_id = ?1
            ORDER BY m.merit_total DESC
            LIMIT ?2
            "#
        )?;

        let rows = stmt.query_map(params![room_id, limit as i32], |row| {
            let user_id: String = row.get(0)?;
            let merit_total: i64 = row.get(1)?;

            // 检查是否有装备的称号
            let title_id: Option<i64> = row.get(2)?;
            let equipped_title = if let Some(id) = title_id {
                Some(Title {
                    id,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    icon: row.get(5)?,
                    condition_kind: row.get::<_, String>(6)?.parse().unwrap_or(ConditionKind::TotalMerit),
                    condition_value: row.get(7)?,
                    rarity: row.get::<_, String>(8)?.parse().unwrap_or(Rarity::Common),
                })
            } else {
                None
            };

            Ok(LeaderboardEntry {
                user_id,
                merit_total,
                equipped_title,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }

    // ==================== 称号操作 ====================

    /// 获取所有称号
    pub fn get_all_titles(&self) -> Result<Vec<Title>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, condition_kind, condition_value, rarity FROM titles ORDER BY rarity, id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Title {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                condition_kind: row.get::<_, String>(4)?.parse().unwrap_or(ConditionKind::TotalMerit),
                condition_value: row.get(5)?,
                rarity: row.get::<_, String>(6)?.parse().unwrap_or(Rarity::Common),
            })
        })?;

        let mut titles = Vec::new();
        for row in rows {
            titles.push(row?);
        }

        Ok(titles)
    }

    /// 获取用户称号列表
    pub fn get_user_titles(&self, user_id: &str, room_id: &str) -> Result<Vec<UserTitle>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT t.id, t.name, t.description, t.icon, t.condition_kind, t.condition_value, t.rarity,
                   ut.equipped, ut.obtained_at
            FROM titles t
            LEFT JOIN user_titles ut ON t.id = ut.title_id AND ut.user_id = ?1 AND ut.room_id = ?2
            WHERE ut.user_id IS NOT NULL
            ORDER BY t.rarity, t.id
            "#
        )?;

        let rows = stmt.query_map(params![user_id, room_id], |row| {
            Ok(UserTitle {
                title: Title {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    condition_kind: row.get::<_, String>(4)?.parse().unwrap_or(ConditionKind::TotalMerit),
                    condition_value: row.get(5)?,
                    rarity: row.get::<_, String>(6)?.parse().unwrap_or(Rarity::Common),
                },
                equipped: row.get::<_, i32>(7)? != 0,
                obtained_at: row.get::<_, Option<String>>(8)?.and_then(|s| s.parse().ok()),
            })
        })?;

        let mut titles = Vec::new();
        for row in rows {
            titles.push(row?);
        }

        Ok(titles)
    }

    /// 解锁称号
    pub fn unlock_title(&self, user_id: &str, room_id: &str, title_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "INSERT OR IGNORE INTO user_titles (user_id, room_id, title_id) VALUES (?1, ?2, ?3)",
            params![user_id, room_id, title_id],
        )?;
        Ok(rows_affected > 0)
    }

    /// 装备称号
    pub fn equip_title(&self, user_id: &str, room_id: &str, title_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // 先取消已装备的称号
        conn.execute(
            "UPDATE user_titles SET equipped = 0 WHERE user_id = ?1 AND room_id = ?2",
            params![user_id, room_id],
        )?;

        // 装备新称号
        let rows_affected = conn.execute(
            "UPDATE user_titles SET equipped = 1 WHERE user_id = ?1 AND room_id = ?2 AND title_id = ?3",
            params![user_id, room_id, title_id],
        )?;

        Ok(rows_affected > 0)
    }

    /// 检查并解锁符合条件的称号
    pub fn check_and_unlock_titles(&self, record: &MeritRecord) -> Result<Vec<Title>> {
        let all_titles = self.get_all_titles()?;
        let mut unlocked = Vec::new();

        for title in all_titles {
            let should_unlock = match title.condition_kind {
                ConditionKind::TotalMerit => record.merit_total >= title.condition_value,
                ConditionKind::DailyHits => record.hits_today >= title.condition_value as i32,
                ConditionKind::Combo => record.max_combo >= title.condition_value as i32,
                ConditionKind::CriticalHits => record.critical_count >= title.condition_value,
                ConditionKind::ConsecutiveDays => record.consecutive_days >= title.condition_value as i32,
            };

            if should_unlock {
                if self.unlock_title(&record.user_id, &record.room_id, title.id)? {
                    unlocked.push(title);
                }
            }
        }

        Ok(unlocked)
    }

    // ==================== 掉落操作 ====================

    /// 添加掉落物品
    pub fn add_drop(&self, user_id: &str, room_id: &str, name: &str, icon: &str, rarity: &Rarity) -> Result<DropItem> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();

        conn.execute(
            "INSERT INTO drops (user_id, room_id, item_name, item_icon, rarity) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, room_id, name, icon, rarity.to_string()],
        )?;

        let id = conn.last_insert_rowid();

        Ok(DropItem {
            id,
            user_id: user_id.to_string(),
            room_id: room_id.to_string(),
            item_name: name.to_string(),
            item_icon: Some(icon.to_string()),
            rarity: *rarity,
            obtained_at: now,
        })
    }

    /// 获取用户掉落物品
    pub fn get_drops(&self, user_id: &str, room_id: &str) -> Result<Vec<DropItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, room_id, item_name, item_icon, rarity, obtained_at
             FROM drops WHERE user_id = ?1 AND room_id = ?2
             ORDER BY obtained_at DESC
             LIMIT 100"
        )?;

        let rows = stmt.query_map(params![user_id, room_id], |row| {
            let obtained_at_str: String = row.get(6)?;
            let obtained_at = obtained_at_str.parse().unwrap_or_else(|_| Utc::now());

            Ok(DropItem {
                id: row.get(0)?,
                user_id: row.get(1)?,
                room_id: row.get(2)?,
                item_name: row.get(3)?,
                item_icon: row.get(4)?,
                rarity: row.get::<_, String>(5)?.parse().unwrap_or(Rarity::Common),
                obtained_at,
            })
        })?;

        let mut drops = Vec::new();
        for row in rows {
            drops.push(row?);
        }

        Ok(drops)
    }

    /// 统计用户掉落物品数量
    pub fn count_drops(&self, user_id: &str, room_id: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM drops WHERE user_id = ?1 AND room_id = ?2",
            params![user_id, room_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}