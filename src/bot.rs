use anyhow::Result;
use matrix_sdk::{
    Client, LoopCtrl, config::SyncSettings, ruma::events::room::member::StrippedRoomMemberEvent,
};
use tracing::info;

use crate::ai_service::AiService;
use crate::config::Config;
use crate::event_handler::{EventHandler, handle_invite};
use crate::store::{Database, PersonaStore};

/// Matrix AI 机器人主结构体。
///
/// 封装了 Matrix 客户端和事件处理器，负责：
/// - 初始化 Matrix 连接和登录
/// - 注册事件处理器
/// - 管理同步循环和优雅关闭
///
/// # Example
///
/// ```no_run
/// use aether_matrix::bot::Bot;
/// use aether_matrix::config::Config;
///
/// async fn run_bot() -> anyhow::Result<()> {
///     let config = Config::from_env()?;
///     let bot = Bot::new(config).await?;
///     bot.run().await
/// }
/// ```
pub struct Bot {
    /// Matrix 客户端实例
    client: Client,
    /// 消息事件处理器
    handler: EventHandler<AiService>,
}

impl Bot {
    /// 从配置创建并初始化 Bot。
    ///
    /// 执行以下步骤：
    /// 1. 创建 Matrix 客户端
    /// 2. 检查是否存在已保存的会话
    /// 3. 如无会话则执行登录
    /// 4. 创建 AI 服务和事件处理器
    ///
    /// # Arguments
    ///
    /// * `config` - 机器人配置
    ///
    /// # Returns
    ///
    /// 成功时返回初始化完成的 `Bot` 实例。
    ///
    /// # Errors
    ///
    /// 当以下情况发生时返回错误：
    /// - Matrix 客户端构建失败
    /// - 登录失败
    /// - 获取用户 ID 失败
    pub async fn new(config: Config) -> Result<Self> {
        // 创建 Matrix 客户端
        let mut client_builder = Client::builder()
            .homeserver_url(&config.matrix_homeserver)
            .sqlite_store(&config.store_path, None);

        // 配置代理（如果需要）
        if let Some(proxy_url) = &config.proxy {
            info!("使用代理: {}", proxy_url);
            client_builder = client_builder.proxy(proxy_url);
        }

        let client = client_builder.build().await?;

        // 检查是否已有有效会话（避免重复登录）
        // 使用 SQLite 存储后，会话状态会持久化，重启无需重新登录
        if client.session_meta().is_some() {
            info!("检测到已存在的会话，跳过登录");
        } else {
            info!("正在登录 Matrix...");

            let mut login_builder = client
                .matrix_auth()
                .login_username(&config.matrix_username, &config.matrix_password)
                .initial_device_display_name(&config.device_display_name);

            // 如果配置了设备ID，使用它以保持设备一致性
            // 避免每次重启都创建新设备，导致设备列表膨胀
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

        // 初始化数据库和 PersonaStore
        // 失败时降级运行，仅禁用 Persona 功能，不影响核心 AI 对话
        let persona_store = match Database::new(&config.db_path) {
            Ok(db) => {
                info!("数据库初始化成功: {}", config.db_path);
                let store = PersonaStore::new(db.conn().clone());
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

        let ai_service = AiService::new(&config);

        let handler = EventHandler::new(
            ai_service,
            user_id.to_owned(),
            client.clone(),
            &config,
            persona_store,
        );

        Ok(Self { client, handler })
    }

    /// 运行 Bot 主循环。
    ///
    /// 执行以下操作：
    /// 1. 注册邀请和消息事件处理器
    /// 2. 启动 Ctrl+C 信号监听
    /// 3. 开始 Matrix 同步循环
    /// 4. 收到关闭信号后优雅退出
    ///
    /// # Returns
    ///
    /// 成功退出时返回 `Ok(())`，失败时返回错误。
    ///
    /// # Graceful Shutdown
    ///
    /// 当收到 `SIGINT` (Ctrl+C) 信号时，Bot 会：
    /// 1. 停止接收新事件
    /// 2. 完成当前同步周期
    /// 3. 优雅退出
    pub async fn run(self) -> Result<()> {
        // 注册邀请事件处理器：自动接受房间邀请
        self.client.add_event_handler(
            |ev: StrippedRoomMemberEvent, client: Client, room: matrix_sdk::Room| async move {
                if let Err(e) = handle_invite(ev, client, room).await {
                    tracing::error!("处理邀请失败: {}", e);
                }
            },
        );

        // 注册消息事件处理器：处理用户消息
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

        // 创建关闭信号通道
        // 使用 watch channel 而非 mpsc，因为需要广播关闭状态给多个消费者
        // watch channel 允许多个消费者同时观察同一个值的变化
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // 启动信号监听任务（独立线程处理 Ctrl+C）
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

        // 开始同步循环
        // 使用回调检查关闭状态，实现优雅退出
        // 每次同步周期都会检查 shutdown_rx，收到信号后立即停止
        self.client
            .sync_with_result_callback(SyncSettings::new(), move |_result| {
                let rx = shutdown_rx.clone();
                async move {
                    // 检查是否收到关闭信号
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
