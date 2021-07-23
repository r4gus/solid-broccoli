use rocket::form::{Form, Strict};
use rocket::serde::{Deserialize, Serialize, json::{Json, json, Value}};
use regex::Regex;
use argon2;
use crate::auth::generate_salt;

use rocket_sync_db_pools::diesel;
use self::diesel::{prelude::*, PgConnection, QueryResult};
use crate::models::*;
use crate::schema::*;
use crate::Db;

#[derive(FromForm, Debug)]
pub struct UserUpdateForm<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

#[derive(FromForm, Debug)]
pub struct UserUpdatePwForm<'a> {
    pub password1: &'a str,
    pub password2: &'a str,
    pub old: &'a str,
}

/// Update the user profile based on the submitted values.
///
/// The function will return a json string as response to indicate if the update
/// has been successfull or not. The response contains a `status` (`ok` or `error`)
/// and a `message`.
#[post("/update/<id>", data = "<form>")]
pub async fn update_user(user: &User, id: i32, form: Form<Strict<UserUpdateForm<'_>>>, conn: Db) -> Value {
    println!("{:?}", form); 

    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    ).unwrap();

    // Only update the user profile if the submitted email is well formed and
    // the id matches the signed up users id.
    if (id != user.id) {
        json!({"status": "error", "message": "Unauthorized request"})
    } else if (form.username.len() < 1) {
        json!({"status": "error", "message": "Invalid username"})
    } else if (!email_regex.is_match(form.email)) {
        json!({"status": "error", "message": "Invalid email"})
    } else {
        let username = form.username.to_string();
        let email = form.email.to_string();
        let first_name = form.first_name.to_string();
        let last_name = form.last_name.to_string();

        // Update the given user
        let result: QueryResult<usize> = conn.run(move |c| {
            let target = users::table.filter(users::id.eq(id));
            diesel::update(target).set((
                users::username.eq(username),
                users::email.eq(email),
                users::first_name.eq(first_name),
                users::last_name.eq(last_name),
            )).execute(c)
        }).await;

        match result {
            Ok(_) => json!({"status": "ok", "message": "User profile successfully updated"}),
            Err(e) => json!({"status": "error", "message": e.to_string()})
        }
    } 
}

#[post("/update/password/<id>", data = "<form>")]
pub async fn update_user_pw(user: &User, id: i32, form: Form<Strict<UserUpdatePwForm<'_>>>, conn: Db) -> Value {
    const PW_MIN_SIZE: usize = 5;
    
    // Verify user via submitted password
    let matches = match argon2::verify_encoded(&user.password_hash, form.old.as_ref()) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("Password update verification failed: {}", e);
            false
        }
    };

    if (id != user.id || !matches) {
        json!({"status": "error", "message": "Unauthorized request"})
    } else if (form.password1 != form.password2) {
        json!({"status": "error", "message": "Passwords don't match"})
    } else if (form.password1.len() < PW_MIN_SIZE) {
        json!({
            "status": "error", 
            "message": format!("Password too short. At least {} chars required", PW_MIN_SIZE)
        })
    } else {
        let password_hash = argon2::hash_encoded(
            form.password1.as_ref(),
            generate_salt(15).as_ref(),
            &argon2::Config::default()
            ).unwrap();

        let result: QueryResult<usize> = conn.run(move |c| {
            let target = users::table.filter(users::id.eq(id));
            diesel::update(target).set((
                users::password_hash.eq(password_hash),
            )).execute(c)
        }).await;

        match result {
            Ok(_) => json!({"status": "ok", "message": "Password successfully updated"}),
            Err(e) => json!({"status": "error", "message": e.to_string()})
        }
    }
}
