//! RSendMail Core Library
//!
//! 这是 RSendMail 的核心库，提供邮件发送的核心功能。
//! 可以被 CLI 和 GUI 应用共享使用。

pub mod anonymizer;
pub mod config;
pub mod mailer;
pub mod stats;

// 重新导出主要类型
pub use anonymizer::EmailAnonymizer;
pub use config::{Config, ProcessMode};
pub use mailer::Mailer;
pub use stats::Stats;
