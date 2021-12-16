# Flash Messages for `actix-web`

Web applications sometimes need to show a **one-time notification** to the user - e.g. an error message after having failed to login.  
These notifications are commonly called **flash messages**.

`actix-web-flash-messages` provides a framework to work with flash messages in `actix-web`, closely modeled after [Django's message framework](https://docs.djangoproject.com/en/3.2/ref/contrib/messages/#module-django.contrib.messages).

```rust
use actix_web::{Responder, HttpResponse, get,http};
use actix_web_flash_messages::{
    FlashMessage, IncomingFlashMessages,
};
use std::fmt::Write;

/// Attach two flash messages to the outgoing response,
/// a redirect.
#[get("/set")]
async fn set() -> impl Responder {
    FlashMessage::info("Hey there!").send();
    FlashMessage::debug("How is it going?").send();
    // Redirect to /show
    HttpResponse::TemporaryRedirect()
        .insert_header((http::header::LOCATION, "/show"))
        .finish()
}

/// Pick up the flash messages attached to the request, showing
/// them to the user via the request body.
#[get("/show")]
async fn show(messages: IncomingFlashMessages) -> impl Responder {
    let mut body = String::new();
    for message in messages.iter() {
        writeln!(body, "{} - {}", message.content(), message.level()).unwrap();
    }
    HttpResponse::Ok().body(body)
}
```

## How to install

Add `actix-web-flash-messages` to your dependencies:

```toml
[dependencies]
# ...
actix-web = "4.0.0-beta.14"
actix-web-flash-messages = "0.2"
```

By default, `actix-web-flash-messages` does not provide any storage backend to receive and send flash messages.  
You can enable:

- a cookie-based one, [`storage::CookieMessageStore`], using the `cookies` feature flag. The cookie store uses a signed cookie to store and retrieve messages;

```toml
[dependencies]
# ...
actix-web-flash-messages = { version = "0.2", features = ["cookies"] }
```

- a session-based one, [`storage::SessionMessageStore`], using the `sessions` feature flag. The session store attaches flash messages to the current session.

```toml
[dependencies]
# ...
actix-web-flash-messages = { version = "0.2", features = ["sessions"] }
```

You can provide a different message store by implementing the [`storage::FlashMessageStore`] trait.

## Examples

You can find examples of application using `actix-web-flash-messages` on GitHub:

- [cookies](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/cookies);
- [cookie-based sessions](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/session-cookie);
- [Redis-based sessions](https://github.com/LukeMathWalker/actix-web-flash-messages/tree/main/examples/session-redis).

## The Structure of a Flash Message

[`FlashMessage`]s are made of a [`Level`] and a string of content.

The message level can be used for filtering and rendering - for example:

- Only show flash messages at `info` level or above in a production environment, while retaining `debug` level messages for local development; 
- Use different colours, in the UI, to display messages (e.g. red for errors, orange for warnings, etc.);

You can build a [`FlashMessage`] via [`FlashMessage::new`] by specifying its content and [`Level`].  
You can also use the shorter level-based constructors - e.g. [`FlashMessage::info`].

## Enabling Flash Messages

To start sending and receiving flash messages you need to register [`FlashMessagesFramework`] as a middleware on your `actix_web`'s `App`:  

```rust
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore};
use actix_web::{HttpServer, App, web};
use actix_web::cookie::Key;

#[actix_web::main]
async fn main() {
    let signing_key = Key::generate(); // This will usually come from configuration!
    let message_store = CookieMessageStore::builder(signing_key).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    
    HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            // [...] your endpoints
    })
    # ;
}
```

You will then be able to:

- extract [`FlashMessage`]s from incoming requests using the [`IncomingFlashMessages`] extractor;
- send [`FlashMessage`]s alongside the outgoing response using [`FlashMessage::send`].

```rust
use actix_web::{Responder, HttpResponse, get};
use actix_web_flash_messages::{
    FlashMessage, IncomingFlashMessages,
};

/// Send a flash messages alongside the outgoing response, a redirect.
#[get("/set")]
async fn set() -> impl Responder {
    FlashMessage::info("Hey there!").send();
    // [...]
    # // Redirect to /show
    # HttpResponse::TemporaryRedirect()
    #     .insert_header((actix_web::http::header::LOCATION, "/show"))
    #     .finish()
}

/// Extract the flash message from the incoming request.
#[get("/show")]
async fn show(_messages: IncomingFlashMessages) -> impl Responder {
    // [...]
    # HttpResponse::Ok()
}
```

## Framework Configuration

There are a few knobs that you can tweak when it comes to [`FlashMessagesFramework`].  
Use [`FlashMessagesFramework::builder`] to get access to its fluent configuration API, built around [`FlashMessagesFrameworkBuilder`].

### Minimum Level

By default, [`FlashMessagesFramework`] will only dispatch messages at `info`-level or above, discarding `debug`-level messages.  
You can change this setting using [`FlashMessagesFrameworkBuilder::minimum_level`].

```rust
use actix_web_flash_messages::{FlashMessagesFramework, Level, storage::CookieMessageStore};
use actix_web::{HttpServer, App, web};

fn get_message_store() -> CookieMessageStore {
    // [...]
    # CookieMessageStore::builder(actix_web::cookie::Key::generate()).build()
}

#[actix_web::main]
async fn main() {
    // Show debug-level messages when developing locally
    let minimum_level = match std::env::var("APP_ENV") {
        Ok(s) if &s == "local" => Level::Debug,
        _ => Level::Info,
    };
    let message_framework = FlashMessagesFramework::builder(get_message_store())
        .minimum_level(minimum_level)
        .build();

    HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            // [...] Your endpoints
    })
    # ;
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
