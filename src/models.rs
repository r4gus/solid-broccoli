use serde::{Serialize, Deserialize};
use super::schema::users;
use rocket_sync_db_pools::diesel;
use self::diesel::{prelude::*, PgConnection, QueryResult};
use rocket::{
    Request,
    request::{self, FromRequest, Outcome},
    outcome::{IntoOutcome},
};
use crate::schema::*;
use super::Db;
use super::auth;

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub house_number: String,
    pub zip: String,
    pub city: String,
    pub phone: String,
    pub img_path: String,
    pub is_admin: bool,
    pub verified: bool,
}

/// User request guard implementation.
///
/// One can use `user: &User` as a request guard. The request guard will either
/// provide a reference to the logged in `User` or forward the request.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r User {
    // This trait implementation requires the `rocket::outcome::IntoOutcome` trait
    // to be into scope.

    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // We store the user within a local cache. The following closure will execute
        // at most once per request, regardless of the number of times the `User` guard
        // is executed.
        let user_result = request.local_cache_async(async {
            // Use the `Db` request guard to get access to the database. Failures are
            // yielded to the calling function.
            let db = request.guard::<Db>().await.succeeded()?;

            // Retrieve the `User` id from the specified session cookie. The cookie must be
            // present otherwise `None` is returned. We need to specify the type of `id`,
            // otherwise the following `if let` expression cant infer it.
            let id: Option<i32> = request.cookies()
                .get_private(super::USER_SESSION_NAME)
                .and_then(|cookie| cookie.value().parse().ok());
            
            // Fetch the user with the given `id` from the database.
            if let Some(id) = id {
                db.run(move |c| {
                    users::table
                        .filter(users::id.eq(id))
                        .get_result(c)
                }).await.ok()
            } else {
                None
            }
        }).await;
        
        // Transform the given `&Option<User>` into an `Option<&User>`.
        // Then convert `Option<&User> into an `Outcome::Success` if its `Some`
        // or an `Outcome::Forward` otherwise.
        user_result.as_ref().or_forward(())
    }
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub house_number: String,
    pub zip: String,
    pub city: String,
    pub phone: String,
    pub img_path: String,
    pub is_admin: bool,
    pub verified: bool,
}

impl NewUser {
    pub fn new(username: &str, email: &str, password: &[u8], is_admin: bool, verified: bool) -> NewUser {
        NewUser {
            username: username.to_string(),
            email: email.to_string(),
            password_hash: argon2::hash_encoded(
                password, 
                auth::generate_salt(15).as_ref(), 
                &argon2::Config::default()).unwrap(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            street: "".to_string(),
            house_number: "".to_string(),
            zip: "".to_string(),
            city: "".to_string(),
            phone: "".to_string(),
            img_path: "".to_string(),
            is_admin: is_admin,
            verified: verified,
        }
    }
}
