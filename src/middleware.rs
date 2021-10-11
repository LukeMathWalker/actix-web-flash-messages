use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

use actix_web::dev::{MessageBody, Service, ServiceRequest, ServiceResponse, Transform};

use crate::builder::FlashMessagesFramework;
use crate::{storage::FlashMessageStore, FlashMessage, Level};
use actix_web::HttpMessage;
use std::sync::Arc;

tokio::task_local! {
    pub(crate) static OUTGOING_MAILBOX: OutgoingMailbox;
}

#[derive(Clone)]
pub(crate) struct OutgoingMailbox {
    pub(crate) messages: RefCell<Vec<FlashMessage>>,
    pub(crate) minimum_level: Level,
}

impl OutgoingMailbox {
    pub(crate) fn new(minimum_level: Level) -> Self {
        Self {
            messages: RefCell::new(vec![]),
            minimum_level,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for FlashMessagesFramework
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = FlashMessagesMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(FlashMessagesMiddleware {
            service,
            storage_backend: self.storage_backend.clone(),
            minimum_level: self.minimum_level,
        }))
    }
}

#[non_exhaustive]
#[doc(hidden)]
pub struct FlashMessagesMiddleware<S> {
    service: S,
    storage_backend: Arc<dyn FlashMessageStore>,
    minimum_level: Level,
}

#[allow(clippy::type_complexity)]
impl<S, B> Service<ServiceRequest> for FlashMessagesMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        req.extensions_mut().insert(self.storage_backend.clone());
        let outgoing_mailbox = OutgoingMailbox::new(self.minimum_level);
        // Working with task-locals in actix-web middlewares is a bit annoying.
        // We need to make the task local value available to the rest of the middleware chain, which
        // generates the `future` which will in turn return us a response.
        // This generation process is synchronous, so we must use `sync_scope`.
        let future =
            OUTGOING_MAILBOX.sync_scope(outgoing_mailbox.clone(), move || self.service.call(req));
        // We can then make the task local value available to the asynchronous execution context
        // using `scope` without losing the messages that might have been recorded by the middleware
        // chain.
        let storage_backend = self.storage_backend.clone();
        Box::pin(OUTGOING_MAILBOX.scope(outgoing_mailbox, async move {
            let response: Result<Self::Response, Self::Error> = future.await;
            response.map(|mut response| {
                OUTGOING_MAILBOX
                    .with(|m| {
                        storage_backend
                            .store(&m.messages.borrow(), response.response_mut().head_mut())
                    })
                    .unwrap();
                response
            })
        }))
    }
}
