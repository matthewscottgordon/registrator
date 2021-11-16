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
    pub async fn add_event(self, event: Event) -> Result<i32, Error> {
        self.run(move |conn| {
            Ok(conn.query(
                "INSERT
                     INTO events (name, time)
                     VALUES ($1, $2)
                     RETURNING id;",
                &[&event.name, &event.datetime],
            )?[0]
                .get(0))
        })
        .await
    }

    pub async fn list_events(self) -> Result<Vec<ObjectWithId<Event>>, Error> {
        self.run(move |conn| {
            Ok(conn
                .query("SELECT id, name, time FROM events;", &[])?
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

    pub async fn get_event(self, id: i32) -> Result<Option<Event>, Error> {
        self.run(move |conn| {
            let rows = conn.query("SELECT name, time FROM events WHERE id = $1;", &[&id])?;
            Ok(if rows.is_empty() {
                None
            } else {
                let row = &rows[0];
                Some(Event {
                    name: row.get(0),
                    datetime: row.get(1),
                })
            })
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
