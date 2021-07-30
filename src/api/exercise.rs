use rocket::form::{Form, Strict};
use rocket::serde::{Deserialize, Serialize, json::{Json, json, Value}};
use rocket::response::{Flash, Redirect};
use rocket::http::{Cookie, CookieJar};
use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};

use rocket_sync_db_pools::diesel;
use self::diesel::{prelude::*, PgConnection, QueryResult};
use crate::models::*;
use crate::schema::*;
use crate::Db;

#[get("/hello/<name>")]
pub fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[derive(FromForm, Debug)]
pub struct RmForm<'a> {
    pub reps: i32,
    pub exercise: &'a str,
    pub weight: f64,
    pub unit: &'a str,
}

#[derive(Serialize)]
pub struct RmStats {
    pub weight: f64,
    pub unit: String,
    pub lifted: chrono::NaiveDateTime,
}

#[post("/insert/rm/<id>", data = "<form>")]
pub async fn insert_rm(user: &User, id: i32, form: Form<Strict<RmForm<'_>>>, conn: Db) -> Value {

    if id != user.id {
        json!({"status": "error", "message": "Unauthorized request"})
    } else {
        let rm = NewRm::new(form.reps, 
                            form.exercise.to_string(), 
                            form.weight, 
                            form.unit.to_string(),
                            id);
        match conn.run(move |c| {
            diesel::insert_into(rms::table)
                .values(rm)
                .execute(c)
        }).await {
            Ok(_) => {
                json!({"status": "ok", "message": format!("{} rep max inserted successfully",
                                                          form.reps)})
            },
            Err(e) => {
                json!({"status": "error", "message": e.to_string()})
            }
        }
    }
}

#[get("/rm/<id>")]
pub async fn get_rms(user: &User, id: i32, conn: Db) -> 
    Result<Json<HashMap<String, HashMap<i32, Vec<RmStats>>>>, Value> {

    if id != user.id {
        Err(json!({"status": "error", "message": "Unauthorized request"}))
    } else {
        let mut map: HashMap<String, HashMap<i32, Vec<RmStats>>> = HashMap::new();
        let user_rms = conn.run(move |c| {
            rms::table
                .filter(rms::uid.eq(id))
                .load::<Rm>(c)
        }).await;
        
        if let Ok(user_rms) = user_rms {
            for rm in user_rms.into_iter() {
                map.entry(rm.exercise)
                    .or_insert(HashMap::new())
                    .entry(rm.reps)
                    .or_insert(Vec::new())
                    .push(RmStats {
                        weight: rm.weight,
                        unit: rm.unit,
                        lifted: rm.lifted
                    });
            }

            Ok(Json(map))
        } else {
            Err(json!({"status": "error", "message": "Database error"}))
        }
    }
}

