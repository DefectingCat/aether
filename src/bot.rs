use anyhow::Result;
use matrix_sdk::{
    Client, LoopCtrl, config::SyncSettings, ruma::events::room::member::StrippedRoomMemberEvent,
};
use tracing::info;

use crate::ai_service::AiService;
use crate::config::Config;
use crate::event_handler::{EventHandler, handle_invite};
use crate::store::{Database, PersonaStore};

/// Matrix AI 机器人
pub struct Bot {
    client: Client,
    handler: EventHandler<AiService>,
}

impl Bot {
    /// 从配置创建 Bot（包含客户端构建和登录）
    pub async fn new(config: Config) -> Result<Self> {
        // 创建 Matrix 客户端
        let mut client_builder = Client::builder()
            .homeserver_url(&config.matrix_homeserver)
            .sqlite_store(&config.store_path, None);

        // 配置代理
        if let Some(proxy_url) = &config.proxy {
            info!("使用代理: {}", proxy_url);
            client_builder = client_builder.proxy(proxy_url);
        }

        let client = client_builder.build().await?;

        // 检查是否已有有效会话
        if client.session_meta().is_some() {
            info!("检测到已存在的会话，跳过登录");
        } else {
            info!("正在登录 Matrix...");

            let mut login_builder = client
                .matrix_auth()
                .login_username(&config.matrix_username, &config.matrix_password)
                .initial_device_display_name(&config.device_display_name);

            // 如果配置了设备ID，使用它
            if let Some(device_id) = &config.matrix_device_id {
                login_builder = login_builder.device_id(device_id.as_str());
                info!("使用配置的设备ID: {}", device_id);
            }

            login_builder.await?;
        }

        let user_id = client
            .user_id()
            .ok_or_else(|| anyhow::anyhow!("登录后无法获取用户ID"))?;
        info!("登录成功: {}", user_id);

        // 初始化数据库
        let persona_store = match Database::new(&config.db_path) {
            Ok(db) => {
                info!("数据库初始化成功: {}", config.db_path);
                let store = PersonaStore::new(db.conn().clone());
                // 初始化内置人设
                if let Err(e) = store.init_builtin_personas() {
                    tracing::warn!("初始化内置人设失败: {}", e);
                }
                Some(store)
            }
            Err(e) => {
                tracing::warn!("数据库初始化失败，Persona 功能将不可用: {}", e);
                None
            }
        };

        // 创建 AI 服务
        let ai_service = AiService::new(&config);

        // 创建事件处理器
        let handler = EventHandler::new(ai_service, user_id.to_owned(), &config, persona_store);

        Ok(Self { client, handler })
    }

    /// 运行 Bot（包含事件注册、优雅关闭、同步循环）
    pub async fn run(self) -> Result<()> {
        // 注册邀请事件处理器
        self.client.add_event_handler(
            |ev: StrippedRoomMemberEvent, client: Client, room: matrix_sdk::Room| async move {
                if let Err(e) = handle_invite(ev, client, room).await {
                    tracing::error!("处理邀请失败: {}", e);
                }
            },
        );

        // 注册消息事件处理器
        self.client.add_event_handler({
            let handler = self.handler;
            let client = self.client.clone();
            move |ev: matrix_sdk::ruma::events::room::message::SyncRoomMessageEvent,
                  room: matrix_sdk::Room| {
                let handler = handler.clone();
                let client = client.clone();
                async move {
                    if let Err(e) = handler.handle_message(ev, room, client).await {
                        tracing::error!("处理消息失败: {}", e);
                    }
                }
            }
        });

        info!("开始同步...");

        // 创建关闭信号通道（使用 watch 通道存储关闭状态）
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // 启动信号监听任务
        tokio::spawn({
            let shutdown_tx = shutdown_tx.clone();
            async move {
                match tokio::signal::ctrl_c().await {
                    Ok(()) => {
                        info!("收到关闭信号，正在停止...");
                        let _ = shutdown_tx.send(true);
                    }
                    Err(e) => {
                        tracing::error!("信号监听错误: {}", e);
                    }
                }
            }
        });

        // 开始同步（使用回调处理错误，实现自动重连和优雅关闭）
        self.client
            .sync_with_result_callback(SyncSettings::new(), move |_result| {
                let rx = shutdown_rx.clone();
                async move {
                    // 检查关闭状态
                    if *rx.borrow() {
                        info!("正在停止同步...");
                        return Ok(LoopCtrl::Break);
                    }
                    Ok(LoopCtrl::Continue)
                }
            })
            .await?;

        info!("机器人已停止");
        Ok(())
    }
}