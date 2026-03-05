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

/// 子命令信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SubCommand {
    /// 子命令名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 使用说明
    pub usage: String,
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
    pub fn commands(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    /// 生成帮助文本（纯文本格式，用于 fallback）
    pub fn generate_help(&self) -> String {
        let mut help = String::from("可用命令:\n\n");

        let mut commands: Vec<_> = self.handlers.iter().collect();
        commands.sort_by_key(|(name, _)| *name);

        for (name, handler) in commands {
            help.push_str(&format!(
                "**!{}** - {}\n",
                name,
                handler.description()
            ));
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