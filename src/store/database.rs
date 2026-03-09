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

        // 迁移 2: 赛博木鱼模块
        let migration_sql = include_str!("../../migrations/20260306000000_muyu.sql");
        conn.execute_batch(migration_sql)?;

        tracing::info!("数据库迁移完成");
        Ok(())
    }

    /// 获取数据库连接
    pub fn conn(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_creates_database_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_string_lossy()
            .to_string();

        let _db = Database::new(&db_path).unwrap();

        assert!(std::path::Path::new(&db_path).exists());
    }

    #[test]
    fn test_new_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("subdir")
            .join("test.db")
            .to_string_lossy()
            .to_string();

        let _db = Database::new(&db_path).unwrap();

        assert!(std::path::Path::new(&db_path).exists());
        assert!(temp_dir.path().join("subdir").exists());
    }

    #[test]
    fn test_new_runs_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_string_lossy()
            .to_string();

        let db = Database::new(&db_path).unwrap();

        let conn = db.conn.lock().unwrap();
        let result: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='personas'",
            [],
            |row| row.get(0),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_conn_returns_arc_mutex() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_string_lossy()
            .to_string();

        let db = Database::new(&db_path).unwrap();

        let conn = db.conn();
        let _guard = conn.lock().unwrap();
    }

    #[test]
    fn test_clone_shares_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir
            .path()
            .join("test.db")
            .to_string_lossy()
            .to_string();

        let db1 = Database::new(&db_path).unwrap();
        let db2 = db1.clone();

        let conn1 = db1.conn.lock().unwrap();
        drop(conn1);
        let conn2 = db2.conn.lock().unwrap();
        drop(conn2);
    }
}
