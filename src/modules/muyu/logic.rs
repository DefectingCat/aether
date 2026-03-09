//! 赛博木鱼游戏逻辑

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use rand::Rng;

use super::models::*;
use super::store::MuyuStore;

/// 敲击冷却时间（毫秒）
const COOLDOWN_MS: u64 = 500;

/// 连击时间窗口（秒）
const COMBO_WINDOW_SECS: i64 = 60;

/// 会心一击概率 (1%)
const CRITICAL_RATE: f64 = 0.01;

/// 普通掉落概率 (10%)
const DROP_RATE: f64 = 0.10;

/// 会心一击掉落概率 (50%)
const CRITICAL_DROP_RATE: f64 = 0.50;

/// 木鱼游戏逻辑
pub struct MuyuLogic {
    store: MuyuStore,
    /// 用户最后敲击时间（防刷限流）
    last_hit_times: DashMap<String, Instant>,
    /// 掉落表
    drop_table: DropTable,
}

impl MuyuLogic {
    /// 创建新的游戏逻辑实例
    pub fn new(store: MuyuStore) -> Self {
        Self {
            store,
            last_hit_times: DashMap::new(),
            drop_table: DropTable::default(),
        }
    }

    /// 执行敲木鱼操作
    pub fn hit(&self, user_id: &str, room_id: &str) -> Result<HitResult> {
        let key = format!("{}:{}", user_id, room_id);

        // 1. 防刷限流检查
        if let Some(last_time) = self.last_hit_times.get(&key) {
            if last_time.elapsed() < Duration::from_millis(COOLDOWN_MS) {
                return Ok(HitResult {
                    merit_gained: 0,
                    merit_total: 0,
                    new_combo: 0,
                    is_critical: false,
                    combo_multiplier: 1.0,
                    dropped_item: None,
                    unlocked_titles: Vec::new(),
                });
            }
        }

        // 更新最后敲击时间
        self.last_hit_times.insert(key.clone(), Instant::now());

        // 2. 获取当前功德记录
        let current_record = self.store.get_merit(user_id, room_id)?;

        // 3. 计算连击
        let (new_combo, should_reset) = self.calculate_combo(&current_record);

        // 如果需要重置连击（超时），先重置
        if should_reset {
            self.store.reset_combo(user_id, room_id)?;
        }

        // 4. 会心一击判定
        let is_critical = self.roll_critical();

        // 5. 计算功德
        let base_merit = if is_critical { 100 } else { 1 };
        let multiplier = self.combo_multiplier(new_combo);
        let merit_gained = ((base_merit as f64) * multiplier) as i64;

        // 6. 更新功德记录
        let updated_record = self.store.update_merit(user_id, room_id, merit_gained, new_combo, is_critical)?;

        // 7. 掉落判定
        let dropped_item = if self.roll_drop(is_critical) {
            let drop_def = self.drop_table.roll();
            Some(self.store.add_drop(user_id, room_id, &drop_def.name, &drop_def.icon, &drop_def.rarity)?)
        } else {
            None
        };

        // 8. 称号解锁检查
        let unlocked_titles = self.store.check_and_unlock_titles(&updated_record)?;

        Ok(HitResult {
            merit_gained,
            merit_total: updated_record.merit_total,
            new_combo,
            is_critical,
            combo_multiplier: multiplier,
            dropped_item,
            unlocked_titles,
        })
    }

    /// 计算连击数
    fn calculate_combo(&self, record: &Option<MeritRecord>) -> (i32, bool) {
        match record {
            None => (1, false),
            Some(r) => {
                // 检查是否在连击窗口内
                let should_reset = if let Some(last_hit) = r.last_hit {
                    let now = Utc::now();
                    let diff = (now - last_hit).num_seconds();
                    diff > COMBO_WINDOW_SECS
                } else {
                    false
                };

                if should_reset {
                    (1, true)
                } else {
                    (r.combo + 1, false)
                }
            }
        }
    }

    /// 计算连击倍率
    fn combo_multiplier(&self, combo: i32) -> f64 {
        if combo >= 20 {
            3.0
        } else if combo >= 5 {
            1.5
        } else {
            1.0
        }
    }

    /// 会心一击判定
    fn roll_critical(&self) -> bool {
        let mut rng = rand::rng();
        rng.random::<f64>() < CRITICAL_RATE
    }

    /// 掉落判定
    fn roll_drop(&self, is_critical: bool) -> bool {
        let mut rng = rand::rng();
        let rate = if is_critical { CRITICAL_DROP_RATE } else { DROP_RATE };
        rng.random::<f64>() < rate
    }
}

/// 掉落表
#[derive(Debug, Clone)]
pub struct DropTable {
    items: Vec<DropDef>,
    cumulative_weights: Vec<f64>,
}

impl Default for DropTable {
    fn default() -> Self {
        let items = vec![
            // Common (权重高 = 常见)
            DropDef { name: "木鱼屑".to_string(), icon: "✨".to_string(), rarity: Rarity::Common, weight: 60.0 },
            DropDef { name: "香灰".to_string(), icon: "💫".to_string(), rarity: Rarity::Common, weight: 60.0 },
            // Rare
            DropDef { name: "佛珠".to_string(), icon: "📿".to_string(), rarity: Rarity::Rare, weight: 25.0 },
            DropDef { name: "莲花".to_string(), icon: "🪷".to_string(), rarity: Rarity::Rare, weight: 25.0 },
            // Epic
            DropDef { name: "金钟".to_string(), icon: "🔔".to_string(), rarity: Rarity::Epic, weight: 12.0 },
            DropDef { name: "菩提叶".to_string(), icon: "🍃".to_string(), rarity: Rarity::Epic, weight: 12.0 },
            // Legendary
            DropDef { name: "舍利子".to_string(), icon: "💎".to_string(), rarity: Rarity::Legendary, weight: 3.0 },
            DropDef { name: "佛光".to_string(), icon: "🌈".to_string(), rarity: Rarity::Legendary, weight: 3.0 },
        ];

        let total_weight: f64 = items.iter().map(|i| i.weight).sum();
        let mut cumulative_weights = Vec::with_capacity(items.len());
        let mut cumulative = 0.0;

        for item in &items {
            cumulative += item.weight / total_weight;
            cumulative_weights.push(cumulative);
        }

        Self { items, cumulative_weights }
    }
}

impl DropTable {
    /// 随机抽取一个掉落物品
    pub fn roll(&self) -> DropDef {
        let mut rng = rand::rng();
        let roll = rng.random::<f64>();

        for (i, &threshold) in self.cumulative_weights.iter().enumerate() {
            if roll < threshold {
                return self.items[i].clone();
            }
        }

        // 兜底返回最后一个
        self.items.last().cloned().unwrap_or_else(|| DropDef {
            name: "木鱼屑".to_string(),
            icon: "✨".to_string(),
            rarity: Rarity::Common,
            weight: 1.0,
        })
    }
}