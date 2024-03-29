#[macro_use] extern crate rocket;

// Работа с документами
pub mod api;
// Callback телеграм бот
pub mod telegram_bot;
// Аккаунт пользователя
pub mod user_account;
// Связь с БД
pub mod db_conn;

use std::path::{Path, PathBuf};
use rocket_sync_db_pools::{database, rusqlite};
use rocket::serde::json::{serde_json, Value};
use once_cell::sync::Lazy;
use rocket::fs::{self, NamedFile, relative};

// Конфиг для путей сохранения аватарок и документов
struct Config {
    path_to_save_docs: String,
    path_to_save_img: String,
    telegram_bot_token: String,
}

impl Config {
    pub fn new() -> Self {

        let config_string = std::fs::read_to_string("config.json").expect("Ошибка стения файла конфига");
        let config_v = serde_json::from_str::<Value>(&config_string).expect("Необходим правильно настроен конфиг");

        let documents_path = config_v["documents"].as_str();
        let images_path = config_v["images"].as_str();
        let tel_bot_token = config_v["bot_token"].as_str();

        if (documents_path, images_path, tel_bot_token) == (None, None, None) {
            panic!("Неверный формат конфига");
        }

        return Config {
            path_to_save_docs: documents_path.unwrap().to_owned(),
            path_to_save_img: images_path.unwrap().to_owned(),
            telegram_bot_token: tel_bot_token.unwrap().to_owned(),
        }
    }
}

static CONFIG: Lazy<Config> = Lazy::new(Config::new);

/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
pub async fn icon() -> Option<rocket::fs::NamedFile> {
    fs::NamedFile::open(Path::new("sochSite\\public\\favicon.ico")).await.ok()
}

/// Главная страница сайта
#[get("/")]
pub async fn index() -> Option<rocket::fs::NamedFile> {
    fs::NamedFile::open("sochSite\\dist\\index.html").await.ok()
}

#[get("/<other_pages>")]
pub async fn index_pages(other_pages: PathBuf) -> Option<rocket::fs::NamedFile> {
    fs::NamedFile::open("sochSite\\dist\\index.html").await.ok()
}

#[get("/<css_path..>")]
pub async fn index_css(css_path: PathBuf) -> Option<rocket::fs::NamedFile> {
    let path = Path::new("sochSite\\dist\\css\\").join(css_path);

    fs::NamedFile::open(path).await.ok()
}

#[get("/<js_path..>")]
pub async fn index_js(js_path: PathBuf) -> Option<rocket::fs::NamedFile> {
    let path = Path::new("sochSite\\dist\\js\\").join(js_path);
    fs::NamedFile::open(path).await.ok()
}

#[get("/<img_path..>")]
pub async fn index_img(img_path: PathBuf) -> Option<rocket::fs::NamedFile> {
    let path = Path::new("sochSite\\dist\\img\\").join(img_path);
    fs::NamedFile::open(path).await.ok()
}

#[catch(404)]
async fn not_found(req: &rocket::Request<'_>) -> Option<NamedFile> {
    fs::NamedFile::open(Path::new("sochSite\\dist\\index.html")).await.ok()
}

// Соединение с базой данных
#[database("rusqlite")]
pub struct Db(rusqlite::Connection);

#[launch]
pub fn rocket() -> _ {
    println!(
        "Путь для сохранения документов: {}\nПуть для сохранения фотографий: {}\n",
        CONFIG.path_to_save_docs,
        CONFIG.path_to_save_img
    );

    rocket::build()
        .mount("/", routes![index, icon])
        .mount("/css", routes![index_css])
        .mount("/js", routes![index_js])
        .mount("/img", routes![index_img])
        .register("/", catchers![not_found])
        .attach(Db::fairing())
        .attach(api::state())
        .attach(user_account::state())
        .attach(telegram_bot::state())
        .manage(
            Config{
                path_to_save_docs: format!("documents{}", std::path::MAIN_SEPARATOR),
                path_to_save_img: format!("avatars{}", std::path::MAIN_SEPARATOR),
                telegram_bot_token: "".to_string(),
            }
        )
}
