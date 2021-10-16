use crate::storage::{FlashMessageStore, LoadError, StoreError};
use crate::FlashMessage;
use actix_session::UserSession;
use actix_web::dev::ResponseHead;
use actix_web::HttpRequest;

pub struct SessionMessageStore {
    key: String,
}

impl FlashMessageStore for SessionMessageStore {
    fn load(&self, request: &HttpRequest) -> Result<Vec<FlashMessage>, LoadError> {
        let session = request.get_session();
        let messages = session
            .get(&self.key)
            .map_err(|e| {
                // This sucks - we are losing all context.
                let e = anyhow::anyhow!("{}", e)
                    .context("Failed to retrieve flash messages from session storage.");
                LoadError::GenericError(e)
            })?
            .unwrap_or_default();
        Ok(messages)
    }

    fn store(
        &self,
        messages: &[FlashMessage],
        request: HttpRequest,
        _response: &mut ResponseHead,
    ) -> Result<(), StoreError> {
        let session = request.get_session();
        session.insert(&self.key, messages).map_err(|e| {
            // This sucks - we are losing all context.
            let e = anyhow::anyhow!("{}", e)
                .context("Failed to retrieve flash messages from session storage.");
            StoreError::GenericError(e)
        })?;
        Ok(())
    }
}
