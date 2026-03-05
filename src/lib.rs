//! # Matrix AI Bot
//!
//! 一个基于 Matrix 协议的 AI 助手机器人，使用 OpenAI 兼容 API 提供聊天功能。
//!
//! ## 功能特性
//!
//! - **流式输出**: 支持打字机效果的实时消息更新
//! - **多会话管理**: 按用户/房间隔离会话历史
//! - **会话持久化**: Matrix SDK 内置 SQLite 存储
//! - **灵活配置**: 支持环境变量和 `.env` 文件配置
//! - **图片理解**: 支持 Vision API 理解用户发送的图片
//!
//! ## 模块结构
//!
//! - [`config`][]: 配置管理，从环境变量加载所有配置项
//! - [`traits`]: AI 服务的 trait 抽象，支持 mock 测试
//! - [`conversation`][]: 多用户/多房间的会话历史管理
//! - [`ai_service`]: OpenAI API 封装，支持普通和流式两种响应模式
//! - [`event_handler`]: Matrix 事件处理，包括邀请和消息事件
//! - [`media`]: 媒体处理，包括图片下载、缩放和 base64 编码
//! - [`bot`][]: 机器人主逻辑，负责初始化和运行

pub mod ai_service;
pub mod bot;
pub mod command;
pub mod config;
pub mod conversation;
pub mod event_handler;
pub mod media;
pub mod modules;
pub mod store;
pub mod traits;
pub mod ui;

pub use traits::ChatStreamResponse;
