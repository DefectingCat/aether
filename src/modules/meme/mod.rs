//! Meme 梗图模块。
//!
//! 提供 `!meme` 命令，使用 KLIPY GIF API 搜索并发送梗图。

mod handlers;
mod klipy;

pub use handlers::MemeHandler;
pub use klipy::KlipyClient;