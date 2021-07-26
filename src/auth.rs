use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::request::FlashMessage;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use rocket::form::{Form, Strict};
use argon2;
use super::Db;
use regex::Regex;
use crate::models::NewUser;
use rocket::serde::{Deserialize, Serialize, json::{Json, json, Value}};

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

/// Validate that the given email address is not malformed.
pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    ).unwrap();

    email_regex.is_match(email)
}

/// Check if the given email address is valid and not already taken.
async fn check_email(email: String, conn: &Db) -> Result<(), String> {
    let cpy = email.clone();

    let taken: bool = conn.run(move |c| {
        users::table
            .filter(users::email.eq(email))
            .get_result::<User>(c)
    }).await.is_ok();

    if (!validate_email(&cpy)) {
        Err(String::from("Malformed email address"))
    } else if (taken) {
        Err(String::from("Email address already taken"))
    } else {
        Ok(())
    }
}

/// Check if the given username is not already taken.
async fn check_username(username: String, conn: &Db) -> bool {
    let taken: bool = conn.run(move |c| {
        users::table
            .filter(users::username.eq(username))
            .get_result::<User>(c)
    }).await.is_ok();

    return taken;
}

#[derive(Debug, FromForm)]
pub struct LoginForm<'r> {
    email: &'r str,
    password: &'r str,
}

#[derive(Debug, FromForm)]
pub struct SignupForm<'r> {
    email: &'r str,
    username: &'r str,
    password1: &'r str,
    password2: &'r str,
}

#[get("/login")]
pub fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    let mut context = Context::new();

    if let Some(ref f) = flash {
        context.parse_flash(f); 
    };

    Template::render("login", &context)
}

#[get("/signup")]
pub fn signup_page(flash: Option<FlashMessage<'_>>) -> Template {
    let mut context = Context::new();

    if let Some(ref f) = flash {
        context.parse_flash(f); 
    };

    Template::render("signup", &context)
}



#[post("/signup", data= "<form>")]
pub async fn signup(form: Form<Strict<SignupForm<'_>>>, conn: Db) -> Flash<Redirect> {

    if let Err(e) = check_email(form.email.to_string(), &conn).await {
        Flash::warning(Redirect::to(uri!(signup)), e)
    } else if (form.password1 != form.password2) {
        Flash::warning(Redirect::to(uri!(signup)), "Passwords do not match")
    } else if (check_username(form.username.to_string(), &conn)).await {
        Flash::warning(Redirect::to(uri!(signup)), "Username already taken")
    } else {
        let mail = form.email.to_string();
        let uname = form.username.to_string();
        let password = form.password1.to_string();

        match conn.run(move |c| {
            diesel::insert_into(users::table)
                .values(NewUser::new(uname.as_ref(), 
                                     mail.as_ref(), 
                                     password.as_ref(), 
                                     false, true))
                .execute(c)
        }).await {
            Ok(_) => {
                Flash::warning(Redirect::to(uri!(login_page)), "Account successfully created")
            },
            Err(e) => {
                Flash::warning(Redirect::to(uri!(login_page)), 
                               format!("Unable to create account: {}", e))
            }
        }

    }
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

#[get("/username/<username>")]
pub async fn username_available(username: &str, conn: Db) -> Value {
    let taken: bool = check_username(username.to_string(), &conn).await;
    if taken {
        json!({"status": "error", "message": "Username already taken"})
    } else if username.len() < 1 {
        json!({"status": "error", "message": "Username too short"})
    } else {
        json!({"status": "ok", "message": "Username available"})
    }
}
