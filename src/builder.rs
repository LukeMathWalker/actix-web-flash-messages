use crate::storage::FlashMessageStore;
use crate::Level;
use std::sync::Arc;

#[derive(Clone)]
/// `actix-web` middleware providing support for sending and receiving [`FlashMessage`]s.
///
/// Use [`FlashMessagesFramework::builder`] to build an instance of [`FlashMessagesFramework`]!
///
/// ```rust
/// use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
/// use actix_web::{HttpServer, App, web, cookie::Key};
///
/// #[actix_web::main]
/// async fn main() {
///     let signing_key = Key::generate(); // This will usually come from configuration!
///     let message_store = CookieMessageStore::builder(signing_key).build();
///     let message_framework = FlashMessagesFramework::builder(message_store).build();
///
///     HttpServer::new(move || {
///         App::new()
///             .wrap(message_framework.clone())
///         // [...] your endpoints
///     })
///     # ;
/// }
/// ```
///
/// [`FlashMessage`]: crate::FlashMessage
pub struct FlashMessagesFramework {
    pub(crate) minimum_level: Level,
    pub(crate) storage_backend: Arc<dyn FlashMessageStore>,
}

impl FlashMessagesFramework {
    /// A fluent API to configure [`FlashMessagesFramework`].
    ///
    /// It takes as input a **message store**, the only required piece of configuration.  
    ///
    /// `actix-web-flash-messages` provides a cookie-based implementation of flash messages, [`CookieMessageStore`],
    /// using a signed cookie to store and retrieve messages.  
    /// You can provide your own custom message store backend by implementing the [`FlashMessageStore`] trait.
    ///
    /// [`CookieMessageStore`]: crate::storage::CookieMessageStore
    pub fn builder<S: FlashMessageStore + 'static>(
        storage_backend: S,
    ) -> FlashMessagesFrameworkBuilder {
        FlashMessagesFrameworkBuilder {
            minimum_level: None,
            storage_backend: Arc::new(storage_backend),
        }
    }
}

/// A fluent builder to construct a [`FlashMessagesFramework`] instance.
pub struct FlashMessagesFrameworkBuilder {
    pub(crate) minimum_level: Option<Level>,
    pub(crate) storage_backend: Arc<dyn FlashMessageStore>,
}

impl FlashMessagesFrameworkBuilder {
    /// By default, [`FlashMessagesFramework`] will only dispatch messages at `info`-level or above, discarding `debug`-level messages.
    /// You can change this behaviour using this method:
    ///
    /// ```rust
    /// use actix_web_flash_messages::{FlashMessagesFramework, Level, storage::CookieMessageStore};
    /// use actix_web::{HttpServer, App, web};
    ///
    /// fn get_message_store() -> CookieMessageStore {
    ///     // [...]
    ///     # CookieMessageStore::builder(actix_web::cookie::Key::generate()).build()
    /// }
    ///
    /// #[actix_web::main]
    /// async fn main() {
    ///     // Show debug-level messages when developing locally
    ///     let minimum_level = match std::env::var("APP_ENV") {
    ///         Ok(s) if &s == "local" => Level::Debug,
    ///         _ => Level::Info,
    ///     };
    ///     let message_framework = FlashMessagesFramework::builder(get_message_store())
    ///         .minimum_level(minimum_level)
    ///         .build();
    ///
    ///     HttpServer::new(move || {
    ///         App::new()
    ///             .wrap(message_framework.clone())
    ///         // [...] Your endpoints
    ///     })
    ///     # ;
    /// }
    /// ```
    pub fn minimum_level(mut self, minimum_level: Level) -> Self {
        self.minimum_level = Some(minimum_level);
        self
    }

    /// Finalise the builder and return a [`FlashMessagesFramework`] instance.
    pub fn build(self) -> FlashMessagesFramework {
        FlashMessagesFramework {
            minimum_level: self.minimum_level.unwrap_or(Level::Info),
            storage_backend: self.storage_backend,
        }
    }
}
