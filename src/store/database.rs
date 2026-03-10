//! 数据库连接和迁移管理。
//!
//! 本模块提供 SQLite 数据库的初始化、连接管理和迁移功能。
//! 使用 `Arc<Mutex<Connection>>` 包装实现线程安全的数据库访问。

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// 数据库管理器，封装 SQLite 连接并提供线程安全的访问。
///
/// `Database` 是所有持久化数据的基础设施，负责管理数据库连接、
/// 执行迁移和提供线程安全的数据库访问接口。
///
/// # 线程安全
///
/// 使用 `Arc<Mutex<Connection>>` 包装数据库连接：
/// - `Arc` 允许在多个线程间共享所有权
/// - `Mutex` 确保同一时间只有一个线程可以访问连接
/// - `Clone` 实现是轻量的，仅增加引用计数
///
/// # 迁移机制
///
/// 创建连接时自动执行数据库迁移：
/// - 从 `migrations/` 目录加载 SQL 迁移脚本
/// - 使用 `include_str!` 在编译时嵌入 SQL
/// - 迁移按文件名字母顺序执行
///
/// # Example
///
/// ```no_run
/// use aether_matrix::store::Database;
///
/// // 创建数据库（自动创建父目录和执行迁移）
/// let db = Database::new("./data/aether.db")?;
///
/// // 获取连接执行查询
/// let conn = db.conn().lock().unwrap();
/// let result: i32 = conn.query_row(
///     "SELECT COUNT(*) FROM personas",
///     [],
///     |row| row.get(0),
/// )?;
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone)]
pub struct Database {
    /// SQLite 数据库连接，使用 `Arc<Mutex>` 包装实现线程安全。
    ///
    /// 通过 [`Database::conn`] 方法获取连接引用，
    /// 使用 `conn.lock().unwrap()` 获取锁后执行数据库操作。
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 创建并初始化数据库连接。
    ///
    /// 初始化流程：
    /// 1. 创建数据库文件的父目录（如果不存在）
    /// 2. 打开或创建 SQLite 数据库文件
    /// 3. 启用外键约束
    /// 4. 执行数据库迁移
    ///
    /// # Arguments
    ///
    /// * `db_path` - 数据库文件路径，支持嵌套路径如 `./data/subdir/aether.db`
    ///
    /// # Returns
    ///
    /// 成功时返回初始化完成的 `Database` 实例，数据库文件已创建并完成迁移。
    ///
    /// # Errors
    ///
    /// 当以下情况发生时返回错误：
    /// - 无法创建数据库目录
    /// - SQLite 连接创建失败
    /// - 外键约束设置失败
    /// - 迁移执行失败
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::store::Database;
    ///
    /// // 创建数据库（自动创建 ./data 目录）
    /// let db = Database::new("./data/aether.db")?;
    ///
    /// // 可以使用嵌套路径
    /// let db2 = Database::new("./data/production/aether.db")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// 获取数据库连接的线程安全引用。
    ///
    /// 返回 `Arc<Mutex<Connection>>` 引用，用于执行数据库操作。
    /// 调用方需要获取锁才能访问实际的 SQLite 连接。
    ///
    /// # Returns
    ///
    /// 数据库连接的 `Arc` 引用，通过 `.lock().unwrap()` 获取锁后使用。
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aether_matrix::store::Database;
    ///
    /// let db = Database::new("./data/aether.db")?;
    ///
    /// // 获取连接并执行查询
    /// let conn = db.conn().lock().unwrap();
    /// let count: i32 = conn.query_row(
    ///     "SELECT COUNT(*) FROM personas",
    ///     [],
    ///     |row| row.get(0),
    /// )?;
    ///
    /// // 锁在 conn 离开作用域时自动释放
    /// drop(conn);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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
