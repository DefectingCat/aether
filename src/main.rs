mod ai_service;
mod bot;
mod command;
mod config;
mod conversation;
mod event_handler;
mod media;
mod modules;
mod store;
mod traits;
mod ui;

use anyhow::Result;
use tracing::info;

use crate::bot::Bot;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = Config::from_env()?;

    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.log_level).expect("Invalid log level"),
        )
        .init();

    info!("配置加载完成");

    // 创建并运行 Bot
    Bot::new(config).await?.run().await
}
