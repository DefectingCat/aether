//! # Matrix AI Bot 入口点
//!
//! 负责初始化日志系统和启动 Bot。
//!
//! # 运行流程
//!
//! 1. 从环境变量加载配置（[`Config::from_env`][]）
//! 2. 初始化 tracing 日志系统
//! 3. 创建 [`Bot`] 实例并启动同步循环
//!
//! # 退出处理
//!
//! 收到 `SIGINT` (Ctrl+C) 信号时，Bot 会优雅退出。

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
    // 配置加载失败时直接返回错误，避免使用默认配置导致运行时问题
    let config = Config::from_env()?;

    // 使用 EnvFilter 支持通过环境变量动态调整日志级别
    // expect 用于快速失败，因为日志系统初始化失败是致命错误
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new(&config.log_level).expect("Invalid log level"),
        )
        .init();

    info!("配置加载完成");

    // 创建并运行 Bot
    Bot::new(config).await?.run().await
}
