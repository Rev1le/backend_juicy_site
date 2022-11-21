mod api;
mod auth;
mod TelegramBot;

use rocket_sync_db_pools::{
    Connection,
    database,
    rusqlite::{
        self,
        params
    }
};

/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("icon_site.ico").await.ok()
}

/// Главная страница сайта
#[get("/")]
async fn index() -> rocket::serde::json::Json<bool> {
    rocket::serde::json::Json(true)
}

#[database("rusqlite")]
pub struct Db(rusqlite::Connection);

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index,icon])
        .attach(Db::fairing())
        .attach(api::stage())
        .attach(auth::stage())
        .attach(TelegramBot::state()
        )
}
