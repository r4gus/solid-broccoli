use rocket_dyn_templates::Template;
use rocket::response::{Flash, Redirect};
use crate::context::Context;
use rocket::request::FlashMessage;
use crate::models::User;

#[get("/dashboard")]
pub fn dashboard(user: &User, flash: Option<FlashMessage<'_>>) -> Result<Template, Flash<Redirect>> {
    let mut context = Context::new();

    if let Some(ref f) = flash {
        context.parse_flash(f); 
    };

    Ok(Template::render("dashboard", &context))
}
