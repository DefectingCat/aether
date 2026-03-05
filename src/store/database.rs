//! 数据库连接和迁移管理

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 数据库管理器
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(db_path: &str) -> Result<Self> {
        // 确保数据库目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建连接
        let conn = Connection::open(db_path)?;

        // 启用外键约束
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // 运行迁移
        db.run_migrations()?;

        tracing::info!("数据库初始化完成: {}", db_path);
        Ok(db)
    }

    /// 运行数据库迁移
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 迁移 1: 初始化表结构
        let migration_sql = include_str!("../../migrations/20260305000000_init.sql");
        conn.execute_batch(migration_sql)?;

        tracing::info!("数据库迁移完成");
        Ok(())
    }

    /// 获取数据库连接
    pub fn conn(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }
}
