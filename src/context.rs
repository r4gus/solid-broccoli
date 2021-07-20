use crate::models::User;
use rocket::request::FlashMessage;
use serde::Serialize;

/// Context used for rendering templates.
#[derive(Serialize)]
pub struct Context<'a> {
    pub user: Option<&'a User>,
    pub flash: Option<(&'a str, &'a str)>,
}

impl<'a> Context<'a> {
    /// Create a fresh context.
    /// All fields are set to `None`.
    pub fn new() -> Self {
        Context {
            user: None,
            flash: None,
        }
    }
    
    /// Create a new `Context` using the given `User` reference.
    pub fn from_user(user: &'a User) -> Self {
        Context {
            user: Some(user),
            flash: None,
        }
    }
    
    /// Parse the given `FlashMessage` reference and add its `kind` and
    /// `value` to `flash` field.
    pub fn parse_flash(&mut self, msg: &'a FlashMessage) {
        self.flash = Some((msg.kind(), msg.message())); 
    }
}
