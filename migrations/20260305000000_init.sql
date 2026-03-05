-- 创建人设表
CREATE TABLE IF NOT EXISTS personas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    avatar_emoji TEXT,
    is_builtin INTEGER DEFAULT 0,
    created_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建房间人设关联表
CREATE TABLE IF NOT EXISTS room_persona (
    room_id TEXT PRIMARY KEY,
    persona_id TEXT REFERENCES personas(id) ON DELETE CASCADE,
    enabled INTEGER DEFAULT 1,
    set_by TEXT,
    set_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建聊天历史表
CREATE TABLE IF NOT EXISTS chat_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引以优化查询
CREATE INDEX IF NOT EXISTS idx_chat_history_room_id ON chat_history(room_id);
CREATE INDEX IF NOT EXISTS idx_chat_history_created_at ON chat_history(created_at);