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

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r User {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user_result = request.local_cache_async(async {
            let db = request.guard::<Db>().await.succeeded().unwrap();
            let id: Option<i32> = request.cookies()
                .get_private(super::USER_SESSION_NAME)
                .and_then(|cookie| cookie.value().parse().ok());

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
        
        user_result.as_ref().or_forward(())
    }
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
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
