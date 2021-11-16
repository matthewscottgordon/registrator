use postgres::error::Error;
use rocket::fairing::AdHoc;
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::postgres;

use super::Event;

#[database("postgres_db")]
pub struct Db(postgres::Client);

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ObjectWithId<T> {
    id: i32,
    object: T,
}

impl Db {
    pub async fn add_event<'a>(self, event: Event) -> Result<u64, Error> {
        self.run(move |conn| {
            conn.execute(
                "INSERT INTO events (name, time) VALUES ($1, $2);",
                &[&event.name, &event.datetime],
            )
        })
        .await
    }

    pub async fn list_events<'a>(self) -> Result<Vec<ObjectWithId<Event>>, Error> {
        self.run(move |conn| {
            Ok(conn
                .query("SELECT id, name, time FROM events", &[])?
                .iter()
                .map(|row| ObjectWithId {
                    id: row.get(0),
                    object: Event {
                        name: row.get(1),
                        datetime: row.get(2),
                    },
                })
                .collect())
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
