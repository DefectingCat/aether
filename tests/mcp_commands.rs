use aether_matrix::command::{CommandHandler, Permission};
use aether_matrix::modules::mcp::McpHandler;
use aether_matrix::traits::AiServiceTrait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_handler_name() {
        let handler = McpHandler::<MockAiService>::new(None, None);
        assert_eq!(handler.name(), "mcp");
    }

    #[tokio::test]
    async fn test_mcp_handler_description() {
        let handler = McpHandler::<MockAiService>::new(None, None);
        assert_eq!(handler.description(), "MCP 服务器管理命令");
    }

    #[tokio::test]
    async fn test_mcp_handler_permission() {
        let handler = McpHandler::<MockAiService>::new(None, None);
        assert_eq!(handler.permission(), Permission::Anyone);
    }

    #[tokio::test]
    async fn test_mcp_handler_usage_contains_expected_subcommands() {
        let handler = McpHandler::<MockAiService>::new(None, None);
        let usage = handler.usage();
        assert!(usage.contains("list"));
        assert!(usage.contains("servers"));
        assert!(usage.contains("reload"));
    }
}

#[cfg(test)]
mod permission_tests {
    use super::*;
    use matrix_sdk::ruma::OwnedUserId;

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::BotOwner > Permission::RoomMod);
        assert!(Permission::RoomMod > Permission::Anyone);
    }
    
    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Anyone.display_name(), "任何人");
        assert_eq!(Permission::RoomMod.display_name(), "房间管理员");
        assert_eq!(Permission::BotOwner.display_name(), "Bot 所有者");
    }

    #[tokio::test]
    async fn test_bot_owner_permission_check_with_valid_owner() {
        let bot_owners = vec!["@owner:matrix.org".to_string()];
        let user_id: OwnedUserId = "@owner:matrix.org".try_into().unwrap();
        
        let has_permission = bot_owners.iter().any(|owner| owner == user_id.as_str());
        assert!(has_permission);
    }

    #[tokio::test]
    async fn test_bot_owner_permission_check_with_invalid_owner() {
        let bot_owners = vec!["@owner:matrix.org".to_string()];
        let user_id: OwnedUserId = "@other:matrix.org".try_into().unwrap();
        
        let has_permission = bot_owners.iter().any(|owner| owner == user_id.as_str());
        assert!(!has_permission);
    }

    #[tokio::test]
    async fn test_bot_owner_permission_check_with_empty_owners() {
        let bot_owners = Vec::<String>::new();
        let user_id: OwnedUserId = "@anyone:matrix.org".try_into().unwrap();
        
        let has_permission = bot_owners.iter().any(|owner| owner == user_id.as_str());
        assert!(!has_permission);
    }
}

#[derive(Clone)]
struct MockAiService;

impl AiServiceTrait for MockAiService {
    async fn chat(&self, _session_id: &str, _prompt: &str) -> anyhow::Result<String> {
        Ok("mock response".to_string())
    }

    async fn chat_with_system(
        &self,
        _session_id: &str,
        _prompt: &str,
        _system_prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        Ok("mock response".to_string())
    }

    async fn reset_conversation(&self, _session_id: &str) {
    }

    async fn chat_stream(
        &self,
        _session_id: &str,
        _prompt: &str,
    ) -> anyhow::Result<(Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>, std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>)> {
        anyhow::bail!("not implemented")
    }

    async fn chat_stream_with_system(
        &self,
        _session_id: &str,
        _prompt: &str,
        _system_prompt: Option<&str>,
    ) -> anyhow::Result<(Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>, std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>)> {
        anyhow::bail!("not implemented")
    }

    async fn chat_with_image(
        &self,
        _session_id: &str,
        _text: &str,
        _image_data_url: &str,
    ) -> anyhow::Result<String> {
        Ok("mock vision response".to_string())
    }

    async fn chat_with_image_stream(
        &self,
        _session_id: &str,
        _text: &str,
        _image_data_url: &str,
    ) -> anyhow::Result<(Arc<tokio::sync::Mutex<aether_matrix::traits::StreamingState>>, std::pin::Pin<Box<dyn futures_util::Stream<Item = anyhow::Result<String>> + Send>>)> {
        anyhow::bail!("not implemented")
    }

    async fn chat_with_tools(
        &self,
        _session_id: &str,
        _prompt: &str,
        _system_prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        Ok("mock tools response".to_string())
    }

    fn mcp_server_manager(&self) -> Option<Arc<RwLock<aether_matrix::mcp::McpServerManager>>> {
        None
    }

    fn inner_mcp_registry(&self) -> Option<Arc<RwLock<aether_matrix::mcp::ToolRegistry>>> {
        None
    }

    async fn list_mcp_tools(&self) -> Vec<aether_matrix::mcp::ToolDefinition> {
        vec![]
    }

    async fn has_tools(&self) -> bool {
        false
    }
}