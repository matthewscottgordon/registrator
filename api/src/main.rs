#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_sync_db_pools;

mod db;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::stage())
        .mount("/", routes![index])
}
