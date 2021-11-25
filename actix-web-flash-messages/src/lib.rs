#![doc = include_str!("../../crate_readme.md")]
mod builder;
mod flash_message;
mod incoming;
mod middleware;
pub mod storage;

pub use builder::{FlashMessagesFramework, FlashMessagesFrameworkBuilder};
pub use flash_message::{FlashMessage, Level};
pub use incoming::IncomingFlashMessages;
pub use middleware::FlashMessagesMiddleware;
