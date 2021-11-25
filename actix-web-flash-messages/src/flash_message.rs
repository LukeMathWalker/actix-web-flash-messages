use crate::middleware::OUTGOING_MAILBOX;
use std::fmt::{Debug, Display, Formatter};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
/// A **one-time** user notification.
///
/// Flash messages are made of a [`Level`] and a string of content.  
/// The message level can be used for filtering and rendering - for example:
///
/// - Only show flash messages at `info` level or above in a production environment, while retaining `debug` level messages for local development;
/// - Use different colours, in the UI, to display messages (e.g. red for errors, orange for warnings, etc.);
///
/// You can build a flash message via [`FlashMessage::new`] by specifying its content and [`Level`].
/// You can also use the shorter level-based constructors - e.g. [`FlashMessage::info`].
pub struct FlashMessage {
    content: String,
    level: Level,
}

impl FlashMessage {
    /// Build a [`FlashMessage`] by specifying its content and [`Level`].
    pub fn new(content: String, level: Level) -> Self {
        Self { content, level }
    }

    /// The string content of this flash message.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// The [`Level`] of this flash message.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Build an info-level [`FlashMessage`] by specifying its content.
    pub fn info<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            level: Level::Info,
        }
    }

    /// Build a debug-level [`FlashMessage`] by specifying its content.
    pub fn debug<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            level: Level::Debug,
        }
    }

    /// Build a success-level [`FlashMessage`] by specifying its content.
    pub fn success<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            level: Level::Success,
        }
    }

    /// Build a warning-level [`FlashMessage`] by specifying its content.
    pub fn warning<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            level: Level::Warning,
        }
    }

    /// Build an error-level [`FlashMessage`] by specifying its content.
    pub fn error<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            level: Level::Error,
        }
    }

    /// Attach this [`FlashMessage`] to the outgoing request.
    ///
    /// The message will be dropped if its [`Level`] is below the minimum level
    /// specified when configuring [`FlashMessagesFramework`] via [`FlashMessagesFrameworkBuilder::minimum_level`].
    ///
    /// This method will **panic** if [`FlashMessagesFramework`] has not been registered as a middleware.
    ///
    /// [`FlashMessagesFramework`]: crate::FlashMessagesFramework
    /// [`FlashMessagesFrameworkBuilder::minimum_level`]: crate::FlashMessagesFrameworkBuilder::minimum_level
    pub fn send(self) {
        let result = OUTGOING_MAILBOX.try_with(|mailbox| {
            if self.level as u8 >= mailbox.minimum_level as u8 {
                mailbox.messages.borrow_mut().push(self);
            }
        });

        if result.is_err() {
            panic!("Failed to send flash message!\n\
                To use `FlashMessages::send` you need to add `FlashMessageFramework` as a middleware \
                on your `actix-web` application using `wrap`. Check out `actix-web-flash-messages`'s documentation for more details.")
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, PartialOrd, Eq)]
/// The severity level of a [`FlashMessage`].
///
/// Levels can be used for filtering and rendering - for example:
///
/// - Only show flash messages at `info` level or above in a production environment, while retaining `debug` level messages for local development;
/// - Use different colours, in the UI, to display messages (e.g. red for errors, orange for warnings, etc.).
pub enum Level {
    /// Development-related messages. Often ignored in a production environment.
    Debug = 0,
    /// Informational messages for the user - e.g. "Your last login was two days ago".
    Info = 1,
    /// Positive feedback after an action was successful - e.g. "You logged in successfully!".
    Success = 2,
    /// Notifying the user about an action that they must take imminently to prevent an error in the future.
    Warning = 3,
    /// An action was **not** successful - e.g. "The provided login credentials are invalid".
    Error = 4,
}

impl Debug for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", level_to_str(self))
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", level_to_str(self))
    }
}

fn level_to_str(l: &Level) -> &'static str {
    match l {
        Level::Debug => "debug",
        Level::Info => "info",
        Level::Success => "success",
        Level::Warning => "warning",
        Level::Error => "error",
    }
}
