use serde::{Serialize, Deserialize};
use super::schema::users;

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
