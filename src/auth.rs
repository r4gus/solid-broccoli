use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::request::FlashMessage;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use rocket::form::{Form, Strict};
use argon2;
use super::Db;

use rocket_sync_db_pools::diesel;
use self::diesel::{prelude::*, PgConnection, QueryResult};
use crate::models::*;
use crate::schema::*;
use crate::context::Context;

/// Generate a random salt for passwords.
///
/// The function only consideres ascii characters.
///
/// # Param
/// * `len` - Requested length of the salt in chars (1 Byte).
pub fn generate_salt(len: usize) -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    let mut rng = rand::thread_rng();

    let salt: String = (0..len).map(|_| {
        let idx = rng.gen_range(0..CHARSET.len());
        CHARSET[idx] as char
    }).collect();

    salt
}

#[derive(Debug, FromForm)]
pub struct LoginForm<'r> {
    email: &'r str,
    password: &'r str,
}

#[get("/login")]
pub fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    let mut context = Context::new();
    //let mut context: HashMap<&str, &str> = HashMap::new();

    if let Some(ref f) = flash {
        context.parse_flash(f); 
    };

    Template::render("login", &context)
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<Strict<LoginForm<'_>>>, cookies: &CookieJar<'_>, conn: Db) -> Flash<Redirect> {
    let email = form.email.to_string();

    // Query the user with the given email.
    let user: Result<User, diesel::result::Error> = conn.run(move |c| {
        users::table
        .filter(users::email.eq(email))
        .get_result(c)
    }).await;
    
    // If 
    match user {
        Ok(u) => {
            if !u.verified {
                return Flash::warning(Redirect::to(uri!(login)),
                    "Your account hasn't been validated yet!");
            }

            match argon2::verify_encoded(&u.password_hash, form.password.as_ref()) {
                // Argon2 compared the password to the hash from the database, now
                // we need to check if it has matched or not.
                Ok(matches) => {
                    if matches { // valid password
                        cookies.add_private(
                            Cookie::new(super::USER_SESSION_NAME, format!("{}", u.id)));
                        return Flash::success(
                            Redirect::to(uri!(super::app::dashboard)), "Login successful");
                    } else { // invalid password
                        return Flash::warning(Redirect::to(uri!(login)), 
                                              "Invalid username or password");
                    }
                },
                Err(e) => {
                    // This shouldn't be called and would indicate a problem with Argon2 itself.
                    eprintln!("Argon2 decryption error: {}", e);
                    return Flash::error(Redirect::to(uri!(login)), "Unexpected server error");
                }
            }
        },
        Err(e) => {
            eprintln!("Unable to fetch requested user from database: {}", e);
            Flash::warning(Redirect::to(uri!(login)), "Invalid username or password")
        }
    }
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(super::USER_SESSION_NAME));
    Flash::success(Redirect::to(uri!(login)), "Successfully logged out.")
}
