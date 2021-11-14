use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;
use rocket_sync_db_pools::postgres;

#[database("postgres_db")]
struct Db(postgres::Client);

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    Db::get_one(&rocket)
        .await
        .expect("database mounted")
        .run(|conn| {
            conn.execute(
                "CREATE TABLE registrations (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    email TEXT,
                    phone TEXT NOT NULL
                )",
                &[],
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
