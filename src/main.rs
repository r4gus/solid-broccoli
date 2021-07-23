#[macro_use] extern crate rocket; // import macros globaly using macro_use
#[macro_use] extern crate rocket_sync_db_pools;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate diesel;
#[macro_use] extern crate regex;

#[cfg(test)] mod test;

mod auth;
mod schema;
mod models;
mod context;
mod app;
mod api;

use rocket::{
    Rocket, Build,
    fs::{FileServer, relative},
    fairing::AdHoc
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database};
use dotenv::dotenv;
use argon2;
use models::NewUser;
use schema::users;
use diesel::{prelude::*, PgConnection};

const USER_SESSION_NAME: &str = "ontrack_uid";

#[database("postgres_db")]
pub struct Db(PgConnection);

/// Run all migrations of the `migrations` directory.
/// This function is meant to be called as AdHoc fairing
/// on startup.
async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    // Embed diesel migrations into binary.
    //
    // This macro defines a `embedded_migrations` module that
    // contains a function named `run` which runs the migrations
    // in the specified directory.
    diesel_migrations::embed_migrations!("migrations");
    
    // Get a database connection.
    let conn = Db::get_one(&rocket).await.expect("database connection");

    // Run all pending migrations.
    conn.run(|c| embedded_migrations::run(c)).await.expect("diesel migrations");

    // Insert a default admin on startup.
    dotenv().ok();
    let mail = std::env::var("ADMIN_EMAIL");
    let pw = std::env::var("ADMIN_PWD");

    if pw.is_ok() && mail.is_ok() {
        let admin = models::NewUser::new(
            "admin", mail.unwrap().as_ref(), pw.unwrap().as_ref(), true, true); 
        if let Err(e) = conn.run(move |c| {
            diesel::insert_into(users::table)
                .values(admin)
                .execute(c)
        }).await {
            eprintln!("Unable to create default admin: {}", e); 
        };
    }

    rocket
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![auth::login_page, auth::login, auth::logout, 
               app::dashboard, app::dashboard_forward])
        .mount("/api/user", routes![api::user::update_user, api::user::update_user_pw])
        // Serve static files relative to the crates root.
        .mount("/static", FileServer::from(relative!("static")))
        // Allow templates as return type
        .attach(Template::fairing())
        // Attach database to application
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
}
