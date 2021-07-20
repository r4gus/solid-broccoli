use super::rocket;
use rocket::local::blocking::{Client, LocalResponse};
use rocket::http::{Status, Cookie, ContentType};
use super::auth;

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

