use actix_web::cookie::Key;
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use actix_web_flash_messages::storage::CookieMessageStore;
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

fn build_message_framework(signing_key: Key) -> FlashMessagesFramework {
    let message_store = CookieMessageStore::builder(signing_key).build();

    // Show debug-level messages when developing locally
    let minimum_level = match std::env::var("APP_ENV") {
        Ok(s) if &s == "local" => Level::Debug,
        _ => Level::Info,
    };
    FlashMessagesFramework::builder(message_store)
        .minimum_level(minimum_level)
        .build()
}

#[actix_web::main]
async fn main() {
    // This will usually come from configuration!
    let signing_key = Key::generate();
    let message_framework = build_message_framework(signing_key);

    HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .route("/show", web::get().to(show))
            .route("/set", web::get().to(set))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
}
