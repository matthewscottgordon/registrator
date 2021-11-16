#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

use chrono::{DateTime, Utc};
use rocket::response::Debug;
use rocket::serde::{de, json::Json, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

mod db;
use db::ObjectWithId;

fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,
    S::Err: Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

fn serialize_to_str<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&datetime.to_rfc3339())
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Event {
    name: String,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "serialize_to_str"
    )]
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

#[get("/events")]
async fn list_events(
    db: db::Db,
) -> Result<Json<Vec<ObjectWithId<Event>>>, Debug<postgres::error::Error>> {
    Ok(Json(db.list_events().await?))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .mount("/", routes![add_event, list_events])
}
