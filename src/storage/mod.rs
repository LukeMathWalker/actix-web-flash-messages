//! Pluggable storage backends for flash messages.
mod interface;

#[cfg(feature = "cookies")]
pub use cookies::{CookieMessageStore, CookieMessageStoreBuilder};
#[cfg(feature = "cookies")]
mod cookies;

pub use interface::{FlashMessageStore, LoadError, StoreError};

#[cfg(feature = "sessions")]
mod sessions;
#[cfg(feature = "sessions")]
pub use sessions::SessionMessageStore;
