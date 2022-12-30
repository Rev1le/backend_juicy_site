#[macro_use] extern crate rocket;

// Работа с документами
pub mod api;
// Авторизация на сайте
pub mod auth;
// Callback телеграм бот
pub mod telegram_bot;

use rocket_sync_db_pools::{database, rusqlite};
use rocket::serde::json::{serde_json, Value};
use once_cell::sync::Lazy;

pub struct Config {
    path_to_save_docs: String,
    path_to_save_img: String,
}

static CONFIG: Lazy<Config> = Lazy::new(|| {

    match std::fs::read_to_string("config.json") {
        Ok(data) => {
            let Ok(v) = serde_json::from_str::<Value>(&data) else {
                panic!("Неверный формат конфига");
            };

            let documents_path = v["documents"].as_str();
            let images_path = v["images"].as_str();

            if documents_path == None || images_path == None {
                panic!("Неверный формат конфига");
            }

            return Config {
                path_to_save_docs: documents_path.unwrap().to_owned(),
                path_to_save_img: images_path.unwrap().to_owned(),
            }
        }
        Err(_) => panic!("Ошибка стения файла конфига")
    }
});

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

    println!("Путь для сохранения документов: {}\nПуть для сохранения фотографий: {}\n", CONFIG.path_to_save_docs, CONFIG.path_to_save_img);

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
