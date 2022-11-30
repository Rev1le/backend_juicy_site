#[macro_use] extern crate rocket;
// Работа с документами
pub mod api;
// Авторизация на сайте
pub mod auth;
// Callback телеграм бот
pub mod telegram_bot;

use rocket_sync_db_pools::{database, rusqlite};


/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
pub async fn icon() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("icon_site.ico").await.ok()
}

/// Главная страница сайта
#[get("/")]
pub async fn index() -> rocket::serde::json::Json<bool> {
    rocket::serde::json::Json(true)
}

// Соединение с базой данных
#[database("rusqlite")]
pub struct Db(rusqlite::Connection);

#[launch]
pub fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, icon])
        .attach(Db::fairing())
        .attach(api::stage())
        .attach(auth::stage())
        .attach(telegram_bot::state())
}
