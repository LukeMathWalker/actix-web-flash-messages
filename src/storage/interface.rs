use crate::FlashMessage;
use actix_web::dev::ResponseHead;
use actix_web::HttpRequest;

/// The interface to retrieve and dispatch flash messages.
///
/// `actix-web-flash-messages` provides a cookie-based implementation of flash messages, [`CookieMessageStore`],
/// using a signed cookie to store and retrieve messages.  
/// You can provide your own custom message store backend by implementing this trait.
///
/// [`CookieMessageStore`]: crate::storage::CookieMessageStore
pub trait FlashMessageStore: Send + Sync {
    /// Extract flash messages from an incoming request.
    fn load(&self, request: &HttpRequest) -> Result<Vec<FlashMessage>, LoadError>;

    /// Attach flash messages to an outgoing response.
    fn store(
        &self,
        messages: &[FlashMessage],
        response: &mut ResponseHead,
    ) -> Result<(), StoreError>;
}

#[derive(thiserror::Error, Debug)]
/// Possible failures modes for [`FlashMessageStore::load`].
pub enum LoadError {
    #[error("Failed to deserialize incoming flash messages")]
    DeserializationError(#[source] anyhow::Error),
    #[error("The content of incoming flash messages failed a cryptographic integrity check (e.g. signature verification)")]
    IntegrityCheckFailed(#[source] anyhow::Error),
    #[error("Something went wrong when extracting incoming flash messages")]
    GenericError(#[source] anyhow::Error),
}

/// Possible failures modes for [`FlashMessageStore::store`].
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("Failed to serialize outgoing flash messages")]
    SerializationError(#[source] anyhow::Error),
    #[error("Outgoing flash messages, when serialised, exceeded the store size limit")]
    SizeLimitExceeded(#[source] anyhow::Error),
    #[error("Something went wrong when flushing outgoing flash messages")]
    GenericError(#[source] anyhow::Error),
}
