mod ai_service;
mod config;
mod conversation;
mod event_handler;

use anyhow::Result;
use matrix_sdk::{
    Client, config::SyncSettings, ruma::events::room::member::StrippedRoomMemberEvent,
};
use tracing::info;

use crate::ai_service::AiService;
use crate::config::Config;
use crate::event_handler::EventHandler;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("info").expect("Invalid env filter"),
        )
        .init();

    // 加载配置
    let config = Config::from_env()?;
    info!("配置加载完成");

    // 创建 Matrix 客户端
    let client = Client::builder()
        .homeserver_url(&config.matrix_homeserver)
        .sqlite_store("./store", None)
        .build()
        .await?;

    info!("正在登录 Matrix...");

    // 登录
    client
        .matrix_auth()
        .login_username(&config.matrix_username, &config.matrix_password)
        .initial_device_display_name(&config.device_display_name)
        .await?;

    let user_id = client.user_id().unwrap();
    info!("登录成功: {}", user_id);

    // 创建 AI 服务
    let ai_service = AiService::new(&config);

    // 创建事件处理器
    let handler = EventHandler::new(ai_service, user_id.to_owned(), &config);

    // 注册邀请事件处理器 - StrippedState 事件会自动提供 Room
    client.add_event_handler(
        |ev: StrippedRoomMemberEvent, client: Client, room: matrix_sdk::Room| async move {
            if let Err(e) = EventHandler::handle_invite(ev, client, room).await {
                tracing::error!("处理邀请失败: {}", e);
            }
        },
    );

    // 注册消息事件处理器
    client.add_event_handler({
        let handler = handler;
        move |ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
              room: matrix_sdk::Room| {
            let handler = handler.clone();
            async move {
                if let Err(e) = handler.handle_message(ev, room).await {
                    tracing::error!("处理消息失败: {}", e);
                }
            }
        }
    });

    info!("开始同步...");

    // 开始同步
    client.sync(SyncSettings::new()).await?;

    Ok(())
}
