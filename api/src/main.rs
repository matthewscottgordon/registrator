#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use chrono::{DateTime, Utc};
use rocket::response::Debug;
use rocket::serde::{de, json::Json, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

mod db;

fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,      // Required for S::from_str...
    S::Err: Display, // Required for .map_err(de::Error::custom)
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Event {
    name: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    datetime: DateTime<Utc>,
}

#[post("/events/add", data = "<event>")]
async fn add_event(
    db: db::Db,
    event: Json<Event>,
) -> Result<Json<u64>, Debug<postgres::error::Error>> {
    let count = db.add_event(event.into_inner()).await?;
    Ok(Json(count))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .mount("/", routes![index, add_event])
}
