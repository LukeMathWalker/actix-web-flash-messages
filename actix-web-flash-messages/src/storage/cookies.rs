use crate::storage::interface::{FlashMessageStore, LoadError, StoreError};
use crate::FlashMessage;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::cookie::{CookieJar, Key};
use actix_web::dev::ResponseHead;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::HttpRequest;
use anyhow::Context;
use percent_encoding::{percent_encode, AsciiSet};

/// A cookie-based implementation of flash messages.
///
/// [`CookieMessageStore`] uses a signed cookie to store and retrieve [`FlashMessage`]s.  
///
/// Use [`CookieMessageStore::builder`] to build an instance of [`CookieMessageStore`]!
///
/// You can find an example of an application using [`CookieMessageStore`]
/// [on GitHub](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/cookies).
pub struct CookieMessageStore {
    cookie_name: String,
    signing_key: Key,
    bytes_size_limit: u32,
    secure: bool,
}

/// A fluent builder to construct a [`CookieMessageStore`] instance.
pub struct CookieMessageStoreBuilder {
    cookie_name: Option<String>,
    signing_key: Key,
    bytes_size_limit: Option<u32>,
    secure: Option<bool>,
}

impl CookieMessageStore {
    /// A fluent API to configure [`CookieMessageStore`].
    ///
    /// It takes as input a **signing key**, the only required piece of configuration.  
    /// The cookie used to store flash messages is signed - this ensures that flash messages
    /// were authored by the application and were not tampered with.  
    pub fn builder(signing_key: Key) -> CookieMessageStoreBuilder {
        CookieMessageStoreBuilder {
            cookie_name: None,
            signing_key,
            bytes_size_limit: None,
            secure: None,
        }
    }

    /// Serialise and percent-encode outgoing flash messages.
    ///
    /// FIX(luca): we are using an intermediate JSON representation because `serde_urlencoded` does not
    /// support serialising sequences of structs.
    /// This is extremely wasteful in terms of storage space - quite problematic given that:
    /// - this payload is sent over the wire;
    /// - cookies cannot be bigger than 4096 bytes.
    fn encode(&self, messages: &[FlashMessage]) -> Result<Cookie<'_>, StoreError> {
        let serialised = serde_json::to_string(messages)
            .context("Failed to serialise flash messages to JSON.")
            .map_err(StoreError::SerializationError)?;

        // Sign the payload **before** doing percent-encoding
        let mut cookie_jar = CookieJar::new();
        cookie_jar
            .signed_mut(&self.signing_key)
            .add(Cookie::new(self.cookie_name.to_owned(), serialised));
        let signed_cookie = cookie_jar.get(&self.cookie_name).unwrap();

        // Then percent-encode the value and set all relevant cookie properties.
        let encoded_value =
            percent_encode(signed_cookie.value().as_bytes(), USERINFO_ENCODE_SET).to_string();
        if encoded_value.len() > self.bytes_size_limit as usize {
            Err(StoreError::SizeLimitExceeded(anyhow::anyhow!(
                "The configured maximum cookie size, in bytes, is {}. The serialised and signed outgoing flash messages are {} bytes long.",
                self.bytes_size_limit,
                encoded_value.len()
            )))
        } else {
            let signed_cookie = Cookie::build(&self.cookie_name, encoded_value)
                .secure(self.secure)
                .http_only(true)
                .same_site(SameSite::Lax)
                // In the future, consider making the `path` configurable - either globally or on a per-endpoint basis
                .path("/")
                .finish();

            Ok(signed_cookie)
        }
    }

    fn decode(&self, cookie: Cookie<'static>) -> Result<Vec<FlashMessage>, LoadError> {
        let mut cookie_jar = CookieJar::new();
        cookie_jar.add_original(cookie);
        if let Some(cookie) = cookie_jar.signed(&self.signing_key).get(&self.cookie_name) {
            let messages = serde_json::from_str(cookie.value()).context(
                "Failed to deserialise the URL-decoded flash messages according to the JSON format",
            ).map_err(LoadError::DeserializationError)?;
            Ok(messages)
        } else {
            Err(LoadError::IntegrityCheckFailed(anyhow::anyhow!(
                "Signature validation failed for the cookie storing incoming flash messages"
            )))
        }
    }
}

impl CookieMessageStoreBuilder {
    /// By default, the cookie used to store messages is named `_flash`.  
    /// You can use `cookie_name` to set the name to a custom value.
    pub fn cookie_name(mut self, name: String) -> Self {
        self.cookie_name = Some(name);
        self
    }

    /// By default, the cookie used to store flash messages is capped at
    /// 2048 bytes.
    ///
    /// This is to ensure [broad cross-browser compatibility](https://www.quora.com/What-Is-The-Maximum-Size-Of-Cookie-In-A-Web-Browser)
    /// while leaving enough room for other cookies in the response.  
    ///
    /// Make sure to research the limits of the browsers you are targeting
    /// before raising this limit.
    pub fn bytes_size_limit(mut self, bytes_size_limit: u32) -> Self {
        self.bytes_size_limit = Some(bytes_size_limit);
        self
    }

    /// By default, secure is set to true.
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = Some(secure);
        self
    }

    /// Finalise the builder and return a [`CookieMessageStore`] instance.
    pub fn build(self) -> CookieMessageStore {
        CookieMessageStore {
            cookie_name: self.cookie_name.unwrap_or_else(|| "_flash".to_string()),
            signing_key: self.signing_key,
            bytes_size_limit: self.bytes_size_limit.unwrap_or(2048),
            secure: self.secure.unwrap_or(true),
        }
    }
}

impl FlashMessageStore for CookieMessageStore {
    fn load(&self, request: &HttpRequest) -> Result<Vec<FlashMessage>, LoadError> {
        if let Some(cookie) = request.cookie(&self.cookie_name) {
            Ok(self.decode(cookie)?)
        } else {
            Ok(vec![])
        }
    }

    fn store(
        &self,
        messages: &[FlashMessage],
        _request: HttpRequest,
        response_head: &mut ResponseHead,
    ) -> Result<(), StoreError> {
        if !messages.is_empty() {
            let cookie = self.encode(messages)?;

            response_head
                .add_cookie(&cookie)
                .context("Failed to add the flash message cookie to the response")
                .map_err(StoreError::GenericError)?;
        } else {
            // Make sure to clear up previous flash messages!
            // No need to do this on the other if-branch because we are overwriting
            // any pre-existing cookie with a new value.
            let removal_cookie = Cookie::build(self.cookie_name.clone(), "")
                .max_age(time::Duration::seconds(0))
                // In the future, consider making the `path` configurable - either globally or on a per-endpoint basis
                .path("/")
                .finish();
            response_head
                .add_cookie(&removal_cookie)
                .context("Failed to add 'removal cookie' for flash message storage to the response")
                .map_err(StoreError::GenericError)?;
        }
        Ok(())
    }
}

/// [Spec](https://url.spec.whatwg.org/#fragment-percent-encode-set)
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

/// [Spec](https://url.spec.whatwg.org/#path-percent-encode-set)
const PATH_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'#').add(b'?').add(b'{').add(b'}');

/// [Spec](https://url.spec.whatwg.org/#userinfo-percent-encode-set)
const USERINFO_ENCODE_SET: &AsciiSet = &PATH_ENCODE_SET
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b'%');

/// FIX(luca): we are using an extension trait to provide cookie-related methods on `ResponseHead`.
/// This is necessary because `actix-web` only provides `add_cookie`/`del_cookie` on `HttpResponse`,
/// but using `HttpResponse` as input type for `load` in `MessageStore` would force us to add a
/// generic parameter that would suddenly make `MessageStore` no longer object-safe - a.k.a.
/// we cannot use `Arc<dyn MessageStore>`.
///
/// The implementations of `add_cookie` and `del_cookie` are copy-pasted from `actix-web`.
/// These two methods on `ResponseHead` can probably be added upstream.
trait ResponseHeadExt {
    fn add_cookie(&mut self, cookie: &Cookie) -> Result<(), anyhow::Error>;
}

impl ResponseHeadExt for ResponseHead {
    fn add_cookie(&mut self, cookie: &Cookie) -> Result<(), anyhow::Error> {
        HeaderValue::from_str(&cookie.to_string())
            .map(|c| {
                self.headers_mut().append(header::SET_COOKIE, c);
            })
            .map_err(|e| e.into())
    }
}
