use actix_web::cookie::Key;
use actix_web::web::resource;
use actix_web::{web, App, HttpResponse, Responder};
use actix_web_flash_messages::{FlashMessage, FlashMessagesFramework, IncomingFlashMessages};
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
        .insert_header((actix_web::http::header::LOCATION, "/show"))
        .finish()
}

#[cfg(feature = "sessions")]
mod cookies {
    use super::*;
    use actix_web_flash_messages::storage::CookieMessageStore;

    #[actix_rt::test]
    async fn test_flash_messages_workflow_with_cookies() {
        let cookie_name = "my-custom-cookie-name".to_string();
        let cookie_store = CookieMessageStore::builder(Key::generate())
            .cookie_name(cookie_name.clone())
            .build();
        let messages_framework = FlashMessagesFramework::builder(cookie_store).build();
        let app = actix_web::test::init_service(
            App::new()
                .wrap(messages_framework)
                .service(resource("/set").route(web::get().to(set)))
                .service(resource("/show").route(web::get().to(show))),
        )
        .await;

        // Step 0:  GET /show
        // No flash messages have been set - the response should be setting the flash cookie
        // with max_age set to 0.
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/show")
                .to_request(),
        )
        .await;
        let cookies = resp.response().cookies().collect::<Vec<_>>();
        assert_eq!(cookies.len(), 1);
        let cookie = cookies.first().unwrap();
        assert_eq!(cookie.name(), cookie_name);
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age(), Some(time::Duration::seconds(0)));

        let body_length = actix_web::test::read_body(resp).await.len();
        assert_eq!(body_length, 0);

        // Step 1:  GET /set
        // One flash message is passed in the response via cookies - the debug-level message
        // is ignored.
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get().uri("/set").to_request(),
        )
        .await;
        let flash_cookie = resp
            .response()
            .cookies()
            .find(|c| c.name() == cookie_name)
            .unwrap();

        // Step 2:  GET /show
        // The flash message is correctly read from the cookie and returned as part of the
        // body.
        // The response contains a directive to delete the flash cookie (one-time usage).
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/show")
                .cookie(flash_cookie)
                .to_request(),
        )
        .await;
        let cookies = resp.response().cookies().collect::<Vec<_>>();
        assert_eq!(cookies.len(), 1);
        let cookie = cookies.first().unwrap();
        assert_eq!(cookie.name(), cookie_name);
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.max_age(), Some(time::Duration::seconds(0)));

        let body_bytes = actix_web::test::read_body(resp).await;
        let body = std::str::from_utf8(&body_bytes).unwrap();
        assert_eq!(body, "Hey there! - info\n");
    }
}

#[cfg(feature = "sessions")]
mod sessions {
    use super::*;
    use actix_session::{storage::CookieSessionStore, SessionMiddleware};
    use actix_web_flash_messages::storage::SessionMessageStore;

    #[actix_rt::test]
    async fn test_flash_messages_workflow_with_session_cookies() {
        let cookie_name = "_session";
        let master_key = Key::generate();
        let session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), master_key)
                .cookie_name("_session".to_string())
                .cookie_http_only(true)
                .cookie_secure(true)
                .build();
        let app = actix_web::test::init_service(
            App::new()
                .wrap(FlashMessagesFramework::builder(SessionMessageStore::default()).build())
                .wrap(session_middleware)
                .service(resource("/set").route(web::get().to(set)))
                .service(resource("/show").route(web::get().to(show))),
        )
        .await;

        // Step 0:  GET /show
        // No flash messages have been set - the response should not be setting a session cookie.
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/show")
                .to_request(),
        )
        .await;
        assert_eq!(resp.response().cookies().count(), 0);

        let body_length = actix_web::test::read_body(resp).await.len();
        assert_eq!(body_length, 0);

        // Step 1:  GET /set
        // One flash message is passed in the response via the session cookie -
        // the debug-level message is ignored.
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get().uri("/set").to_request(),
        )
        .await;
        let session_cookie = resp
            .response()
            .cookies()
            .find(|c| c.name() == cookie_name)
            .unwrap();

        // Step 2:  GET /show
        // The flash message is correctly read from the session cookie and returned
        // as part of the body.
        // The response contains a directive to set the session cookie to a value
        // that does not contain any flash message (one-time usage).
        let resp = actix_web::test::call_service(
            &app,
            actix_web::test::TestRequest::get()
                .uri("/show")
                .cookie(session_cookie)
                .to_request(),
        )
        .await;
        let cookies = resp.response().cookies().collect::<Vec<_>>();
        assert_eq!(cookies.len(), 1);
        let cookie = cookies.first().unwrap();
        assert_eq!(cookie.name(), cookie_name);
        // Ignoring the signature
        assert!(!cookie.value().is_empty());

        let body_bytes = actix_web::test::read_body(resp).await;
        let body = std::str::from_utf8(&body_bytes).unwrap();
        assert_eq!(body, "Hey there! - info\n");
    }
}
