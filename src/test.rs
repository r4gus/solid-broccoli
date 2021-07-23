use super::rocket;
use rocket::local::blocking::{Client, LocalResponse};
use rocket::http::{Status, Cookie, ContentType, uri::Origin};
use super::auth;
use super::app;
use super::api;
use dotenv::dotenv;
use std::panic;
use regex::Regex;

use rocket_sync_db_pools::diesel;
use self::diesel::{prelude::*, PgConnection, QueryResult};
use crate::models::*;
use crate::schema::*;
use crate::Db;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn insert_test_user(conn: &PgConnection, user: &NewUser) -> User {
    diesel::insert_into(users::table)
                .values(user)
                .get_result(conn)
                .expect("Insert test user")
}

fn delete_test_user(conn: &PgConnection, user: &User) {
    diesel::delete(users::table.filter(users::id.eq(user.id))).execute(conn);
}

fn get_test_user(conn: &PgConnection, id: i32) -> User {
    users::table.filter(users::id.eq(id)).get_result(conn).expect("Load test user")
}

fn setup() {

}

fn teardown() {
    // Purge database
    let conn = establish_connection();
    diesel::delete(users::table).execute(&conn);
}

fn run_test<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
{
    setup();
    let result = panic::catch_unwind(|| {
        test()
    });
    teardown();
    assert!(result.is_ok())
}

/// Fetch a session cookie from the given response.
/// The function will return a session cookie on success and `None` otherwise.
fn user_id_cookie(response: &LocalResponse<'_>) -> Option<Cookie<'static>> {
    let cookie = response.headers()
        .get("Set-Cookie")
        .filter(|v| v.starts_with(super::USER_SESSION_NAME))
        .nth(0)
        .and_then(|val| Cookie::parse_encoded(val).ok());

    cookie.map(|c| c.into_owned())
}

/// Login to web app.
/// The function will return a session cookie on success and `None` otherwise.
fn login(client: &Client, user: &str, pass: &str) -> Option<Cookie<'static>> {
    let response = client.post(uri!(auth::login))
        .header(ContentType::Form)
        .body(format!("email={}&password={}", user, pass))
        .dispatch();

    user_id_cookie(&response)
}

#[test]
fn view_login_page() {
    let client = Client::tracked(rocket()).expect("valid rocket instance");
    let mut response = client.get("/login").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.into_string().unwrap().contains("sign in"));
}

#[test]
fn login_fails() {
    let client = Client::tracked(rocket()).unwrap();
    assert!(login(&client, "admin@admin.com", "secre!").is_none());
    assert!(login(&client, "admin@admin.co", "secret").is_none());
    assert!(login(&client, "david@example.de", "secret").is_none());
}

#[test]
fn login_succeeds() {
    let client = Client::tracked(rocket()).unwrap();
    assert!(login(&client, "admin@admin.com", "secret").is_some());
}

#[test]
fn login_logout() {
    let client = Client::tracked(rocket()).unwrap();
    let session_cookie = login(&client, "admin@admin.com", "secret").expect("logged in");

    // Ensure we're logged in
    let response = client.get(uri!(app::dashboard)).cookie(session_cookie.clone()).dispatch();
    assert_eq!(response.status(), Status::Ok);
    
    // Logout
    let response = client.get(uri!(auth::logout)).cookie(session_cookie).dispatch();
    let cookie =  user_id_cookie(&response).expect("logout cookie");
    assert!(cookie.value().is_empty());

    // The user should be redirected to the login page
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(response.headers().get_one("Location").unwrap(), &uri!(auth::login));
    
    // Try to access the dashboard without being logged in
    let response = client.get(uri!(app::dashboard)).cookie(cookie.clone()).dispatch();
    assert_eq!(response.status(), Status::SeeOther);
    assert_eq!(response.headers().get_one("Location").unwrap(), &uri!(auth::login));
}

fn post_req<'a>(client: &'a Client, payload: &'a str, url: Origin<'a>) -> LocalResponse<'a> {
    client.post(url)
        .header(ContentType::Form)
        .body(payload)
        .dispatch()
}

#[test]
fn user_profile_update() {
    run_test(|| {
        let rocket = rocket();
        let conn = establish_connection();

        let max = NewUser::new("maxi", "max@mustermann.de", b"secret", false, true);

        let max: User = insert_test_user(&conn, &max);

        let client = Client::tracked(rocket).unwrap();
        
        // Try to make an unauthorized request
        let f = format!("username={}&email={}&first_name={}&last_name={}", &max.username, &max.email, "Max", "Mustermann");
        let response = post_req(&client, &f, uri!("/api/user/", api::user::update_user(max.id)));
        assert_eq!(response.status(), Status::NotFound);
        
        // Login and try to update another user without permission
        let session_cookie = login(&client, &max.email, "secret").expect("logged in");
        let response = client.post(uri!("/api/user", api::user::update_user(id = max.id + 1)))
            .header(ContentType::Form)
            .body(&f)
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Unauthorized request"));

        // Try to insert a malformed email address
        let response = client.post(uri!("/api/user", api::user::update_user(id = max.id)))
            .header(ContentType::Form)
            .body(format!("username={}&email={}&first_name={}&last_name={}", &max.username, "max.com", "Max", "Mustermann"))
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Invalid email"));

        // Try to submit a empty username
        let response = client.post(uri!("/api/user", api::user::update_user(id = max.id)))
            .header(ContentType::Form)
            .body(format!("username={}&email={}&first_name={}&last_name={}", "", "max@mustermann.de", "Max", "Mustermann"))
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Invalid username"));

        // Finaly update profile
        let response = client.post(uri!("/api/user", api::user::update_user(id = max.id)))
            .header(ContentType::Form)
            .body(&f)
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("User profile successfully updated"));

        let max = get_test_user(&conn, max.id);

        assert_eq!("Max", &max.first_name);
        assert_eq!("Mustermann", &max.last_name);
    })
}

#[test]
fn update_password() {
    run_test(|| {
        let conn = establish_connection();
        let max = NewUser::new("maxi", "max@mustermann.de", b"secret", false, true);
        let max: User = insert_test_user(&conn, &max);

        let client = Client::tracked(rocket()).unwrap();

        // Try to make an unauthorized request
        let f = format!("password1={}&password2={}&old={}", "hic", "hic", "secret");
        let response = 
            post_req(&client, &f, uri!("/api/user/", api::user::update_user_pw(max.id)));
        assert_eq!(response.status(), Status::NotFound);

        // Login and try to update with two passwords that don't match
        let session_cookie = login(&client, &max.email, "secret").expect("logged in");
        let response = client.post(uri!("/api/user", api::user::update_user_pw(id = max.id)))
            .header(ContentType::Form)
            .body(format!("password1={}&password2={}&old={}", "hicrhodos", "hicsalta", "secret"))
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Passwords don't match"));

        // Password is too short
        let response = client.post(uri!("/api/user", api::user::update_user_pw(id = max.id)))
            .header(ContentType::Form)
            .body(&f)
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Password too short"));

        // Old password incorrect
        let response = client.post(uri!("/api/user", api::user::update_user_pw(id = max.id)))
            .header(ContentType::Form)
            .body(format!("password1={}&password2={}&old={}", "hicrhodos", "hicrhodos", "seCret"))
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Unauthorized request"));

        // Update password
        let response = client.post(uri!("/api/user", api::user::update_user_pw(id = max.id)))
            .header(ContentType::Form)
            .body(format!("password1={}&password2={}&old={}", "hicrhodos", "hicrhodos", "secret"))
            .cookie(session_cookie.clone())
            .dispatch();
        let strresp = response.into_string().unwrap();
        assert!(strresp.contains("Password successfully updated"));

        // Logout
        let response = client.get(uri!(auth::logout)).cookie(session_cookie).dispatch();
        let cookie =  user_id_cookie(&response).expect("logout cookie");
        assert!(cookie.value().is_empty());

        // Try to login with old password
        assert!(login(&client, &max.email, "secret").is_none());

        // Try to login with new password
        assert!(login(&client, &max.email, "hicrhodos").is_some());
    })
}





