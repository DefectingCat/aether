use aether_matrix::command::{CommandHandler, Permission};
use aether_matrix::modules::persona::PersonaHandler;
use aether_matrix::store::{Database, PersonaStore};
use tempfile::TempDir;

fn create_test_store() -> (PersonaStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
    let db = Database::new(&db_path).unwrap();
    let store = PersonaStore::new(db.conn().clone());
    store.init_builtin_personas().unwrap();
    (store, temp_dir)
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_persona_handler_name() {
        let (store, _temp_dir) = create_test_store();
        let handler = PersonaHandler::new(store);
        assert_eq!(handler.name(), "persona");
    }

    #[tokio::test]
    async fn test_persona_handler_description() {
        let (store, _temp_dir) = create_test_store();
        let handler = PersonaHandler::new(store);
        assert_eq!(handler.description(), "人设管理命令");
    }

    #[tokio::test]
    async fn test_persona_handler_usage() {
        let (store, _temp_dir) = create_test_store();
        let handler = PersonaHandler::new(store);
        assert_eq!(handler.usage(), "persona <set|list|off|info|create|delete>");
    }

    #[tokio::test]
    async fn test_persona_handler_permission() {
        let (store, _temp_dir) = create_test_store();
        let handler = PersonaHandler::new(store);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_persona_handler_usage_contains_all_subcommands() {
        let (store, _temp_dir) = create_test_store();
        let handler = PersonaHandler::new(store);
        let usage = handler.usage();
        assert!(usage.contains("set"));
        assert!(usage.contains("list"));
        assert!(usage.contains("off"));
        assert!(usage.contains("info"));
        assert!(usage.contains("create"));
        assert!(usage.contains("delete"));
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_command_shows_all_builtin_personas() {
        let (store, _temp_dir) = create_test_store();
        let personas = store.get_all().unwrap();
        
        assert_eq!(personas.len(), 4);
        let actual_ids: Vec<&str> = personas.iter().map(|p| p.id.as_str()).collect();
        assert!(actual_ids.contains(&"sarcastic-dev"));
        assert!(actual_ids.contains(&"cyber-zen"));
        assert!(actual_ids.contains(&"wiki-chan"));
        assert!(actual_ids.contains(&"neko-chan"));
        
        for persona in &personas {
            assert!(persona.is_builtin);
        }
    }

    #[tokio::test]
    async fn test_info_command_returns_correct_details() {
        let (store, _temp_dir) = create_test_store();
        
        let persona = store.get_by_id("sarcastic-dev").unwrap().unwrap();
        assert_eq!(persona.name, "毒舌程序员");
        assert_eq!(persona.avatar_emoji, Some("💻".to_string()));
        assert!(persona.system_prompt.contains("20年经验"));
        assert!(persona.is_builtin);
    }

    #[tokio::test]
    async fn test_set_room_persona_works() {
        let (store, _temp_dir) = create_test_store();
        
        store.set_room_persona("!test:matrix.org", "sarcastic-dev", "@user:matrix.org").unwrap();
        
        let persona = store.get_room_persona("!test:matrix.org").unwrap().unwrap();
        assert_eq!(persona.id, "sarcastic-dev");
        assert_eq!(persona.name, "毒舌程序员");
    }

    #[tokio::test]
    async fn test_disable_room_persona_works() {
        let (store, _temp_dir) = create_test_store();
        
        store.set_room_persona("!test2:matrix.org", "cyber-zen", "@user:matrix.org").unwrap();
        let before = store.get_room_persona("!test2:matrix.org").unwrap();
        assert!(before.is_some());
        
        store.disable_room_persona("!test2:matrix.org").unwrap();
        let after = store.get_room_persona("!test2:matrix.org").unwrap();
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn test_create_custom_persona_works() {
        let (store, _temp_dir) = create_test_store();
        
        let custom_persona = aether_matrix::store::Persona {
            id: "custom-test".to_string(),
            name: "Custom Test".to_string(),
            system_prompt: "Custom test prompt".to_string(),
            avatar_emoji: Some("🎯".to_string()),
            is_builtin: false,
            created_by: Some("@user:matrix.org".to_string()),
        };
        
        store.create_persona(&custom_persona).unwrap();
        
        let retrieved = store.get_by_id("custom-test").unwrap().unwrap();
        assert_eq!(retrieved.name, "Custom Test");
        assert_eq!(retrieved.avatar_emoji, Some("🎯".to_string()));
        assert!(!retrieved.is_builtin);
        assert_eq!(retrieved.created_by, Some("@user:matrix.org".to_string()));
    }

    #[tokio::test]
    async fn test_delete_custom_persona_works() {
        let (store, _temp_dir) = create_test_store();
        
        let custom_persona = aether_matrix::store::Persona {
            id: "to-delete".to_string(),
            name: "To Delete".to_string(),
            system_prompt: "Will be deleted".to_string(),
            avatar_emoji: None,
            is_builtin: false,
            created_by: None,
        };
        
        store.create_persona(&custom_persona).unwrap();
        let exists_before = store.get_by_id("to-delete").unwrap();
        assert!(exists_before.is_some());
        
        let deleted = store.delete_persona("to-delete").unwrap();
        assert!(deleted);
        
        let exists_after = store.get_by_id("to-delete").unwrap();
        assert!(exists_after.is_none());
    }

    #[tokio::test]
    async fn test_cannot_delete_builtin_persona() {
        let (store, _temp_dir) = create_test_store();
        
        let deleted = store.delete_persona("sarcastic-dev").unwrap();
        assert!(!deleted);
        
        let still_exists = store.get_by_id("sarcastic-dev").unwrap();
        assert!(still_exists.is_some());
    }

    #[tokio::test]
    async fn test_get_all_sorts_builtin_first() {
        let (store, _temp_dir) = create_test_store();
        
        let custom_persona = aether_matrix::store::Persona {
            id: "aaa-custom".to_string(),
            name: "AAA Custom".to_string(),
            system_prompt: "Test".to_string(),
            avatar_emoji: None,
            is_builtin: false,
            created_by: None,
        };
        store.create_persona(&custom_persona).unwrap();
        
        let personas = store.get_all().unwrap();
        
        assert!(personas[0].is_builtin);
        assert!(personas[1].is_builtin);
        assert!(personas[2].is_builtin);
        assert!(personas[3].is_builtin);
        assert!(!personas[4].is_builtin);
    }
}