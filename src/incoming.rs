use crate::{storage::FlashMessageStore, FlashMessage};
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpRequest};
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
/// An `actix-web` extractor to retrieve [`FlashMessage`]s attached to an incoming request.
///
/// ```rust
/// use actix_web::{Responder, HttpResponse, get};
/// use actix_web_flash_messages::IncomingFlashMessages;
///
/// #[get("/show")]
/// async fn show(messages: IncomingFlashMessages) -> impl Responder {
///     for message in messages.iter() {
///         println!("{} - {}", message.content(), message.level());
///     }
///     HttpResponse::Ok()
/// }
/// ```
///
/// This method will **panic** if [`FlashMessagesFramework`] has not been registered as a middleware.
///
/// [`FlashMessagesFramework`]: crate::FlashMessagesFramework
pub struct IncomingFlashMessages {
    messages: Vec<FlashMessage>,
}

impl IncomingFlashMessages {
    /// Return an iterator over incoming [`FlashMessage`]s.
    pub fn iter(&self) -> impl Iterator<Item = &FlashMessage> {
        self.messages.iter()
    }
}

impl FromRequest for IncomingFlashMessages {
    type Config = ();
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        std::future::ready(extract_flash_messages(req))
    }
}

fn extract_flash_messages(req: &HttpRequest) -> Result<IncomingFlashMessages, actix_web::Error> {
    let message_store = req.extensions()
        .get::<Arc<dyn FlashMessageStore>>()
        .expect("Failed to retrieve flash messages!\n\
            To use the `IncomingFlashMessages` extractor you need to add `FlashMessageFramework` as a middleware \
            on your `actix-web` application using `wrap`. Check out `actix-web-flash-messages`'s documentation for more details.")
        // Cloning here is necessary in order to drop our reference to the request extensions.
        // Some of the methods on `req` will in turn try to use `req.extensions_mut()`, leading to a borrow
        // panic at runtime due to the usage of interior mutability.
        .to_owned();
    message_store
        .load(req)
        .map(|m| IncomingFlashMessages { messages: m })
        .map_err(|e| {
            actix_web::error::InternalError::new(
                anyhow::Error::new(e).context("Invalid flash cookie"),
                StatusCode::BAD_REQUEST,
            )
            .into()
        })
}
