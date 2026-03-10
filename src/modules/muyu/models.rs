//! 赛博木鱼数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 功德记录
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MeritRecord {
    /// 用户 ID
    pub user_id: String,
    /// 房间 ID
    pub room_id: String,
    /// 累计功德
    pub merit_total: i64,
    /// 今日功德
    pub merit_today: i64,
    /// 今日敲击次数
    pub hits_today: i32,
    /// 最后敲击时间
    pub last_hit: Option<DateTime<Utc>>,
    /// 当前连击数
    pub combo: i32,
    /// 最大连击数
    pub max_combo: i32,
    /// 会心一击次数
    pub critical_count: i64,
    /// 连续打卡天数
    pub consecutive_days: i32,
    /// 最后敲击日期（用于判断是否新的一天，控制 merit_today 重置）
    #[allow(dead_code)]
    pub last_hit_date: Option<chrono::NaiveDate>,
}

/// 称号定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Title {
    /// 称号 ID
    pub id: i64,
    /// 称号名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 解锁条件类型
    pub condition_kind: ConditionKind,
    /// 解锁条件值
    pub condition_value: i64,
    /// 稀有度
    pub rarity: Rarity,
}

/// 解锁条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionKind {
    /// 累计功德
    TotalMerit,
    /// 单日敲击次数
    DailyHits,
    /// 最大连击
    Combo,
    /// 会心一击次数
    CriticalHits,
    /// 连续打卡天数
    ConsecutiveDays,
}

impl std::fmt::Display for ConditionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionKind::TotalMerit => write!(f, "total_merit"),
            ConditionKind::DailyHits => write!(f, "daily_hits"),
            ConditionKind::Combo => write!(f, "combo"),
            ConditionKind::CriticalHits => write!(f, "critical_hits"),
            ConditionKind::ConsecutiveDays => write!(f, "consecutive_days"),
        }
    }
}

impl std::str::FromStr for ConditionKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "total_merit" => Ok(ConditionKind::TotalMerit),
            "daily_hits" => Ok(ConditionKind::DailyHits),
            "combo" => Ok(ConditionKind::Combo),
            "critical_hits" => Ok(ConditionKind::CriticalHits),
            "consecutive_days" => Ok(ConditionKind::ConsecutiveDays),
            _ => Err(format!("Unknown condition kind: {}", s)),
        }
    }
}

/// 稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rarity::Common => write!(f, "common"),
            Rarity::Rare => write!(f, "rare"),
            Rarity::Epic => write!(f, "epic"),
            Rarity::Legendary => write!(f, "legendary"),
        }
    }
}

impl std::str::FromStr for Rarity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "common" => Ok(Rarity::Common),
            "rare" => Ok(Rarity::Rare),
            "epic" => Ok(Rarity::Epic),
            "legendary" => Ok(Rarity::Legendary),
            _ => Err(format!("Unknown rarity: {}", s)),
        }
    }
}

impl Rarity {
    /// 获取显示颜色 (HTML 颜色代码)
    pub fn color(&self) -> &'static str {
        match self {
            Rarity::Common => "#808080",    // 灰色
            Rarity::Rare => "#4a90d9",      // 蓝色
            Rarity::Epic => "#a855f7",      // 紫色
            Rarity::Legendary => "#f0c060", // 金色
        }
    }

    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Rarity::Common => "普通",
            Rarity::Rare => "稀有",
            Rarity::Epic => "史诗",
            Rarity::Legendary => "传说",
        }
    }
}

/// 掉落物品
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DropItem {
    /// 物品 ID（预留：物品使用/交易功能）
    #[allow(dead_code)]
    pub id: i64,
    /// 用户 ID（预留：物品使用/交易功能）
    #[allow(dead_code)]
    pub user_id: String,
    /// 房间 ID（预留：物品使用/交易功能）
    #[allow(dead_code)]
    pub room_id: String,
    /// 物品名称
    pub item_name: String,
    /// 物品图标
    pub item_icon: Option<String>,
    /// 稀有度
    pub rarity: Rarity,
    /// 获取时间（预留：限时活动/成就系统）
    #[allow(dead_code)]
    pub obtained_at: DateTime<Utc>,
}

/// 掉落物品定义（用于掉落表）
#[derive(Debug, Clone)]
pub struct DropDef {
    /// 物品名称
    pub name: String,
    /// 物品图标
    pub icon: String,
    /// 稀有度
    pub rarity: Rarity,
    /// 权重
    pub weight: f64,
}

/// 敲木鱼结果
#[derive(Debug, Clone)]
pub struct HitResult {
    /// 获得的功德
    pub merit_gained: i64,
    /// 新的累计功德
    pub merit_total: i64,
    /// 新的连击数
    pub new_combo: i32,
    /// 是否会心一击
    pub is_critical: bool,
    /// 连击倍率
    pub combo_multiplier: f64,
    /// 掉落的物品
    pub dropped_item: Option<DropItem>,
    /// 新解锁的称号
    pub unlocked_titles: Vec<Title>,
}

/// 用户称号（带装备状态）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserTitle {
    /// 称号信息
    pub title: Title,
    /// 是否已装备
    pub equipped: bool,
    /// 获取时间（预留：限时活动/成就系统）
    #[allow(dead_code)]
    pub obtained_at: Option<DateTime<Utc>>,
}

/// 排行榜条目
#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    /// 用户 ID
    pub user_id: String,
    /// 功德值
    pub merit_total: i64,
    /// 已装备的称号
    pub equipped_title: Option<Title>,
}
