use postgres::error::Error;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::{Build, Rocket};
use rocket_sync_db_pools::postgres;

use super::Event;

#[database("postgres_db")]
pub struct Db(postgres::Client);

impl Db {
    pub async fn add_event<'a>(self, event: Json<Event>) -> Result<u64, Error> {
        self.run(move |conn| {
            conn.execute(
                "INSERT INTO events (name, time) VALUES ($1, $2);",
                &[&event.name, &event.datetime],
            )
        })
        .await
    }
}

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    Db::get_one(&rocket)
        .await
        .expect("database mounted")
        .run(|conn| {
            conn.batch_execute(
                "CREATE TABLE IF NOT EXISTS attendees (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    email TEXT,
                    phone TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS events (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    time TIMESTAMP WITH TIME ZONE NOT NULL
                );

                CREATE TABLE IF NOT EXISTS registrations (
                    attendee_id INTEGER REFERENCES attendees (id),
                    event_id INTEGER REFERENCES events (id),
                    PRIMARY KEY(attendee_id, event_id)
                );",
            )
        })
        .await
        .expect("can init Postgres DB");

    rocket
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Postgres Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Postgres Init", init_db))
    })
}
