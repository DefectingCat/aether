use aether_matrix::command::{CommandHandler, Permission};
use tempfile::TempDir;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

async fn create_test_store(db_path: &std::path::Path) -> aether_matrix::modules::muyu::MuyuStore {
    let conn = Connection::open(db_path).unwrap();
    let conn = Arc::new(Mutex::new(conn));
    
    let conn_lock = conn.lock().unwrap();
    conn_lock.execute_batch(
        "
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

        CREATE TABLE IF NOT EXISTS titles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            icon TEXT,
            condition_kind TEXT NOT NULL CHECK(condition_kind IN ('total_merit', 'daily_hits', 'combo', 'critical_hits', 'consecutive_days')),
            condition_value INTEGER NOT NULL,
            rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary'))
        );

        CREATE TABLE IF NOT EXISTS user_titles (
            user_id TEXT NOT NULL,
            room_id TEXT NOT NULL,
            title_id INTEGER NOT NULL REFERENCES titles(id) ON DELETE CASCADE,
            obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            equipped INTEGER DEFAULT 0,
            PRIMARY KEY (user_id, room_id, title_id)
        );

        CREATE TABLE IF NOT EXISTS drops (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            room_id TEXT NOT NULL,
            item_name TEXT NOT NULL,
            item_icon TEXT,
            rarity TEXT NOT NULL CHECK(rarity IN ('common', 'rare', 'epic', 'legendary')),
            obtained_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        INSERT OR IGNORE INTO titles (name, description, icon, condition_kind, condition_value, rarity) VALUES
            ('初心者', '首次敲击木鱼', '🌱', 'total_merit', 1, 'common'),
            ('虔诚信徒', '累计 100 功德', '🙏', 'total_merit', 100, 'common'),
            ('木鱼狂魔', '单日敲击 50 次', '🥁', 'daily_hits', 50, 'rare'),
            ('连击大师', '达成 20 连击', '💥', 'combo', 20, 'rare'),
            ('会心一击者', '触发 10 次会心', '⚡', 'critical_hits', 10, 'epic');
        ",
    ).unwrap();
    drop(conn_lock);
    
    aether_matrix::modules::muyu::MuyuStore::new(conn)
}

#[cfg(test)]
mod basic_tests {
    use super::*;
    use aether_matrix::modules::muyu::{MuyuHandler, MeritHandler, RankHandler, TitleHandler, BagHandler};

    #[tokio::test]
    async fn test_muyu_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.name(), "木鱼");
    }

    #[tokio::test]
    async fn test_muyu_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.description(), "敲一次木鱼，积累功德");
    }

    #[tokio::test]
    async fn test_muyu_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MuyuHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_merit_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.name(), "功德");
    }

    #[tokio::test]
    async fn test_merit_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.description(), "查看当前功德值");
    }

    #[tokio::test]
    async fn test_merit_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = MeritHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_rank_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.name(), "功德榜");
    }

    #[tokio::test]
    async fn test_rank_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.description(), "查看房间功德排行榜");
    }

    #[tokio::test]
    async fn test_rank_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = RankHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_title_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.name(), "称号");
    }

    #[tokio::test]
    async fn test_title_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.description(), "查看或装备称号");
    }

    #[tokio::test]
    async fn test_title_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = TitleHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_bag_handler_name() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.name(), "背包");
    }

    #[tokio::test]
    async fn test_bag_handler_description() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.description(), "查看掉落物品背包");
    }

    #[tokio::test]
    async fn test_bag_handler_permission() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        let handler = BagHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    #[tokio::test]
    async fn test_merit_accumulation() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let user_id = "@test:example.com";
        let room_id = "!room:example.com";
        
        let initial = store.get_merit(user_id, room_id).unwrap();
        assert!(initial.is_none());
        
        let record = store.update_merit(user_id, room_id, 1, 1, false).unwrap();
        assert_eq!(record.merit_total, 1);
        assert_eq!(record.hits_today, 1);
        assert_eq!(record.combo, 1);
        
        let record2 = store.update_merit(user_id, room_id, 1, 2, false).unwrap();
        assert_eq!(record2.merit_total, 2);
        assert_eq!(record2.hits_today, 2);
        assert_eq!(record2.combo, 2);
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let temp_dir = create_temp_dir();
        let db_path = temp_dir.path().join("test.db");
        let store = create_test_store(&db_path).await;
        
        let room_id = "!room:example.com";
        
        store.update_merit("@user1:example.com", room_id, 100, 1, false).unwrap();
        store.update_merit("@user2:example.com", room_id, 50, 1, false).unwrap();
        store.update_merit("@user3:example.com", room_id, 200, 1, false).unwrap();
        
        let rankings = store.get_leaderboard(room_id, 10).unwrap();
        assert_eq!(rankings.len(), 3);
        
        assert_eq!(rankings[0].user_id, "@user3:example.com");
        assert_eq!(rankings[0].merit_total, 200);
        assert_eq!(rankings[1].user_id, "@user1:example.com");
        assert_eq!(rankings[1].merit_total, 100);
        assert_eq!(rankings[2].user_id, "@user2:example.com");
        assert_eq!(rankings[2].merit_total, 50);
    }
}

#[cfg(test)]
mod ui_tests {
    use aether_matrix::ui::{error, info_card, success, warning, leaderboard};

    #[test]
    fn test_ui_message_formats() {
        let msg = success("Test success");
        assert!(msg.contains("Test success"));
        assert!(msg.contains("✓"));

        let msg = error("Test error");
        assert!(msg.contains("Test error"));
        assert!(msg.contains("✕"));

        let items = vec![("功德", "100")];
        let msg = info_card("功德信息", &items);
        assert!(msg.contains("功德信息"));
        assert!(msg.contains("功德"));
        assert!(msg.contains("100"));

        let msg = warning("敲得太快了");
        assert!(msg.contains("敲得太快了"));
        assert!(msg.contains("⚠"));

        let headers = ["排名", "用户", "功德"];
        let rows = vec![
            vec!["1", "user1", "100"],
            vec!["2", "user2", "50"],
        ];
        let msg = leaderboard("功德排行榜", &headers, &rows);
        assert!(msg.contains("功德排行榜"));
        assert!(msg.contains("user1"));
        assert!(msg.contains("user2"));
    }
}