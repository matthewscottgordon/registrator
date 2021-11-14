use rocket::fairing::AdHoc;
use rocket::{Build, Rocket};
use rocket_sync_db_pools::postgres;

#[database("postgres_db")]
struct Db(postgres::Client);

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
                    time TIMESTAMP NOT NULL
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
