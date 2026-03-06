-- 功德记录表
CREATE TABLE IF NOT EXISTS merit (
    user_id TEXT NOT NULL,
    room_id TEXT NOT NULL,
    merit_total INTEGER DEFAULT 0,
    merit_today INTEGER DEFAULT 0,
    hits_today INTEGER DEFAULT 0,
    last_hit DATETIME,
    combo INTEGER DEFAULT 0,
    max_combo INTEGER DEFAULT 0,
    critical_count INTEGER DEFAULT 0,
    consecutive_days INTEGER DEFAULT 0,
    last_hit_date DATE,
    PRIMARY KEY (user_id, room_id)
);

-- 称号定义表
CREATE TABLE IF NOT EXISTS titles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    icon TEXT,
    condition_kind TEXT NOT NULL CHECK(condition_kind IN ('total_merit', 'daily_hits', 'combo', 'critical_hits', 'consecutive_days')),
    condition_value INTEGER NOT NULL,
    rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary'))
);

-- 用户解锁称号表
CREATE TABLE IF NOT EXISTS user_titles (
    user_id TEXT NOT NULL,
    room_id TEXT NOT NULL,
    title_id INTEGER NOT NULL REFERENCES titles(id) ON DELETE CASCADE,
    obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    equipped INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, room_id, title_id)
);

-- 掉落物品记录表
CREATE TABLE IF NOT EXISTS drops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    room_id TEXT NOT NULL,
    item_name TEXT NOT NULL,
    item_icon TEXT,
    rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary')),
    obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引优化
CREATE INDEX IF NOT EXISTS idx_merit_room_total ON merit(room_id, merit_total DESC);
CREATE INDEX IF NOT EXISTS idx_merit_user_room ON merit(user_id, room_id);
CREATE INDEX IF NOT EXISTS idx_user_titles_user_room ON user_titles(user_id, room_id);
CREATE INDEX IF NOT EXISTS idx_drops_user_room ON drops(user_id, room_id);

-- 内置称号数据
INSERT OR IGNORE INTO titles (name, description, icon, condition_kind, condition_value, rarity) VALUES
    ('初心者', '首次敲击木鱼', '🌱', 'total_merit', 1, 'common'),
    ('虔诚信徒', '累计 100 功德', '🙏', 'total_merit', 100, 'common'),
    ('木鱼狂魔', '单日敲击 50 次', '🥁', 'daily_hits', 50, 'rare'),
    ('连击大师', '达成 20 连击', '💥', 'combo', 20, 'rare'),
    ('会心一击者', '触发 10 次会心', '⚡', 'critical_hits', 10, 'epic'),
    ('赛博和尚', '累计 1000 功德', '🧘', 'total_merit', 1000, 'epic'),
    ('功德满满', '累计 10000 功德', '✨', 'total_merit', 10000, 'legendary'),
    ('赛博罗汉', '连续 7 天敲击', '👁️', 'consecutive_days', 7, 'legendary'),
    ('佛祖转世', '累计 100000 功德', '🌟', 'total_merit', 100000, 'legendary');