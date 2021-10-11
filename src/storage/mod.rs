//! Pluggable storage backends for flash messages.
mod cookies;
mod interface;

pub use cookies::{CookieMessageStore, CookieMessageStoreBuilder};
pub use interface::{FlashMessageStore, LoadError, StoreError};
