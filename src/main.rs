#[macro_use] extern crate rocket;
// Работа с документами
pub mod api;
// Авторизация на сайте
pub mod auth;
// Callback телеграм бот
pub mod telegram_bot;

use rocket_sync_db_pools::{database, rusqlite};

pub struct Config {
    path_to_save_docs: String,
    path_to_save_img: String,
}
// Сделать подгрузку данных из конфига

/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
pub async fn icon() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("avatars/icon_site.ico").await.ok()
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
        .manage(
            // Конфиг для путей сохранения аватарок и документов
            Config{
                path_to_save_docs: format!("documents{}", std::path::MAIN_SEPARATOR),
                path_to_save_img: format!("avatars{}", std::path::MAIN_SEPARATOR),
            }
        )
}
