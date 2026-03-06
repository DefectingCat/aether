//! 命令 Handler 注册表

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::command::context::CommandContext;
use crate::ui;

use super::permission::Permission;

/// 命令处理器 trait
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// 命令名称
    fn name(&self) -> &str;

    /// 命令描述
    fn description(&self) -> &str {
        "暂无描述"
    }

    /// 使用说明
    fn usage(&self) -> &str {
        ""
    }

    /// 所需权限
    fn permission(&self) -> Permission {
        Permission::Anyone
    }

    /// 执行命令
    async fn execute(&self, ctx: &CommandContext<'_>) -> Result<()>;
}

/// 命令注册表
#[derive(Clone)]
pub struct CommandRegistry {
    /// 命令处理器映射
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册命令处理器
    pub fn register(&mut self, handler: Arc<dyn CommandHandler>) {
        self.handlers.insert(handler.name().to_string(), handler);
    }

    /// 获取命令处理器
    pub fn get(&self, name: &str) -> Option<Arc<dyn CommandHandler>> {
        self.handlers.get(name).cloned()
    }

    /// 获取所有命令名称
    #[allow(dead_code)]
    pub fn commands(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    /// 生成帮助文本（纯文本格式，用于 fallback）
    pub fn generate_help(&self) -> String {
        let mut help = String::from("可用命令:\n\n");

        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        for (name, handler) in commands {
            help.push_str(&format!("**!{}** - {}\n", name, handler.description()));
            if !handler.usage().is_empty() {
                help.push_str(&format!("  用法: {}\n", handler.usage()));
            }
            if handler.permission() != Permission::Anyone {
                help.push_str(&format!(
                    "  权限: {}\n",
                    handler.permission().display_name()
                ));
            }
            help.push('\n');
        }

        help
    }

    /// 生成 HTML 帮助菜单
    pub fn generate_help_html(&self) -> String {
        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        // 构建命令列表项 (name, description)
        let cmd_items: Vec<(String, String)> = commands
            .iter()
            .map(|(name, handler)| {
                let cmd_name = format!("!{}", name);
                (cmd_name, handler.description().to_string())
            })
            .collect();

        ui::help_menu(&cmd_items)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::context::CommandContext;

    struct MockHandler {
        name: &'static str,
        desc: &'static str,
        permission: Permission,
    }

    impl MockHandler {
        fn new(name: &'static str, desc: &'static str) -> Self {
            Self {
                name,
                desc,
                permission: Permission::Anyone,
            }
        }

        fn with_permission(name: &'static str, desc: &'static str, permission: Permission) -> Self {
            Self {
                name,
                desc,
                permission,
            }
        }
    }

    #[async_trait]
    impl CommandHandler for MockHandler {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            self.desc
        }

        fn permission(&self) -> Permission {
            self.permission
        }

        async fn execute(&self, _ctx: &CommandContext<'_>) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = CommandRegistry::new();
        assert!(registry.commands().is_empty());
    }

    #[test]
    fn test_register_adds_handler() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        assert_eq!(registry.commands().len(), 1);
        assert!(registry.commands().contains(&"test"));
    }

    #[test]
    fn test_get_returns_registered_handler() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test");
    }

    #[test]
    fn test_get_returns_none_for_unregistered() {
        let registry = CommandRegistry::new();
        let retrieved = registry.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_commands_returns_all_names() {
        let mut registry = CommandRegistry::new();
        let handler1 = Arc::new(MockHandler::new("cmd1", "Command 1"));
        let handler2 = Arc::new(MockHandler::new("cmd2", "Command 2"));

        registry.register(handler1);
        registry.register(handler2);

        let commands = registry.commands();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"cmd1"));
        assert!(commands.contains(&"cmd2"));
    }

    #[test]
    fn test_register_overwrites_duplicate() {
        let mut registry = CommandRegistry::new();
        let handler1 = Arc::new(MockHandler::new("test", "First"));
        let handler2 = Arc::new(MockHandler::new("test", "Second"));

        registry.register(handler1);
        registry.register(handler2);

        assert_eq!(registry.commands().len(), 1);
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.description(), "Second");
    }

    #[test]
    fn test_generate_help_includes_command_name() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("help", "Show help"));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("!help"));
    }

    #[test]
    fn test_generate_help_includes_description() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "This is a test command"));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("This is a test command"));
    }

    #[test]
    fn test_generate_help_includes_permission_when_not_anyone() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::with_permission(
            "admin",
            "Admin command",
            Permission::BotOwner,
        ));

        registry.register(handler);

        let help = registry.generate_help();
        assert!(help.contains("权限"));
    }

    #[test]
    fn test_generate_help_html_valid_html() {
        let mut registry = CommandRegistry::new();
        let handler = Arc::new(MockHandler::new("test", "Test command"));

        registry.register(handler);

        let html = registry.generate_help_html();
        assert!(html.contains("<"));
        assert!(html.contains(">"));
        assert!(html.contains("!test"));
    }
}
