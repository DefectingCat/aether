pub mod ai_service;
pub mod bot;
pub mod config;
pub mod conversation;
pub mod event_handler;
pub mod traits;

// 重新导出常用类型，方便测试使用
pub use traits::{ChatStreamResponse, RoomClient, SendMessageResult};
