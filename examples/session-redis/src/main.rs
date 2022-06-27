use actix_session::storage::RedisActorSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::{Key, SameSite};
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::storage::SessionMessageStore;
use actix_web_flash_messages::{
    FlashMessage, FlashMessagesFramework, IncomingFlashMessages, Level,
};
use std::fmt::Write;

async fn show(messages: IncomingFlashMessages) -> impl Responder {
    let mut body = String::new();
    for message in messages.iter() {
        writeln!(body, "{} - {}", message.content(), message.level()).unwrap();
    }
    HttpResponse::Ok().body(body)
}

async fn set() -> impl Responder {
    FlashMessage::info("Hey there!").send();
    FlashMessage::debug("How is it going?").send();
    HttpResponse::SeeOther()
        .insert_header((http::header::LOCATION, "/show"))
        .finish()
}

fn build_message_framework() -> FlashMessagesFramework {
    // Show debug-level messages when developing locally
    let minimum_level = match std::env::var("APP_ENV") {
        Ok(s) if &s == "local" => Level::Debug,
        _ => Level::Info,
    };
    FlashMessagesFramework::builder(SessionMessageStore::default())
        .minimum_level(minimum_level)
        .build()
}

fn build_session_storage(
    redis_address: String,
    key: Key,
) -> SessionMiddleware<RedisActorSessionStore> {
    SessionMiddleware::builder(RedisActorSessionStore::new(&redis_address), key)
        .cookie_http_only(true)
        .cookie_secure(true)
        .cookie_same_site(SameSite::Strict)
        .build()
}

#[actix_web::main]
async fn main() {
    // This will usually come from configuration!
    let key = Key::generate();

    HttpServer::new(move || {
        App::new()
            // Order is important here - the session middleware must be mounted
            // AFTER the message framework middleware.
            .wrap(build_message_framework())
            // You need a running Redis on port 6379
            // You can use `docker run -d -p 6379:6379 redis`
            .wrap(build_session_storage("127.0.0.1:6379".into(), key.clone()))
            .route("/show", web::get().to(show))
            .route("/set", web::get().to(set))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
}
