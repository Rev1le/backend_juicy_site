use std::path::Path;
use reqwest::get;
use rocket_sync_db_pools::rusqlite::{Connection, OptionalExtension};
use rocket::{serde::json::{Json, Value}, fairing::AdHoc, State};
use uuid::Uuid;

use TgBot_api::bot_methods::TelegramBotMethods;

pub use TgBot_api::types;

use crate::{CONFIG, Db, user_account::StateAuthUser};

pub const TG_API: &str = "https://api.telegram.org/";

pub struct TgBot;
impl TelegramBotMethods for TgBot {}

#[post("/", data="<update_data>")]
async fn get_tg_update<'a>(
    cache: &State<crate::user_account::CacheSessions>,
    db: Db,
    update_data: Json<Value>
) -> Json<bool> {

    println!("{:?}", update_data);

    // Если пришло сообщение
    if let Some(message) = update_data.get("message") {
        check_message(message, &db).await;
    }

    // если бот получил callback сообщение
    if let Some(callback_query) = update_data.get("callback_query") {
        check_callback_query(cache, callback_query).await;
    }

    Json(true)
}

async fn check_message(message: &Value, db: &Db) {

    let chat_id = message["chat"]["id"].as_i64().unwrap();
    let user_id = match message.get("from") {
        Some(user) => user["id"].as_i64().unwrap(),
        _ => return
    };
    let username = match message.get("from") {
        Some(user) => user["first_name"].as_str().unwrap(),
        _ => return
    };
    let message_text = match message.get("text") {
        Some(text) => text.as_str().unwrap(),
        _ => return
    };

    // Является ли сообщение командой
    if Some('/') != message_text.chars().next() {
        println!("This is not command!");
        return;
    }

    let vec_words = message_text
        .split(" ")
        .collect::<Vec<&str>>();

    // Команда регистрации пользователя
    if vec_words[0] == "/reg" && vec_words.len() == 2 {

        let reg_nickname = vec_words[1];
        account_register(username, reg_nickname, user_id, chat_id, db).await;
    }
}

async fn account_register(
    username: &str,
    nickname: &str,
    user_id: i64,
    chat_id: i64,
    db: &Db)
{
    let tmp_nick = nickname.to_owned();

    if let Ok(Some(id)) = db.run(
        move |conn: &mut Connection| {
        conn.query_row(
            "SELECT nickname, tg_id FROM users WHERE nickname=(?)",
            [tmp_nick],
            |row| row.get::<usize, i64>(1)).optional()
        }
    ).await {
        println!("Пользователь зареган с tg_id = {}", id);
        let _ = TgBot::send_api_method(
            "sendMessage",
            &format!("{}{}", &TG_API, &CONFIG.telegram_bot_token),
            &[
                ("chat_id", chat_id.to_string().as_str()),
                ("text", "Аккаунт с таким ником уже зарегестрирован.")
            ]
        ).await;
        return;
    }
    println!("Пользователь не зареган.");

    let user_ava_filename = get_user_avatar(user_id).await;

    let insert_values =[
        username.to_string(),
        nickname.to_string(),
        user_ava_filename,
        "No_Role".to_string(),
        false.to_string(),
        user_id.to_string(),
        Uuid::new_v4().to_string(),
    ];

    db.run(move |conn: &mut Connection| {
        let mut stmt = conn
            .prepare("INSERT INTO users (name, nickname, avatar, role, admin, tg_id, uuid) VALUES(?,?,?,?,?,?,?)")
            .expect("Ошибка при добавлении");
        stmt
            .execute(insert_values)
            .expect("Ошибка при добавлении");
    }).await;
    //inline_keyboard
    TgBot::send_api_method("sendMessage", &CONFIG.telegram_bot_token,
                        &[
            ("chat_id", chat_id.to_string().as_str()),
            ("text", "Пользователь успешно зарегестрирован!!!")
        ]
    ).await.unwrap();
}

pub async fn get_user_avatar(user_id: i64) -> String {
    use reqwest::get;

    let response = TgBot::send_api_method("getUserProfilePhotos", &CONFIG.telegram_bot_token, [("user_id", user_id.to_string())]).await.unwrap();
    let response_json = response;
    let telegram_fileid = &response_json["result"]["photos"][0][1]["file_id"];
    println!("{}", telegram_fileid);

    let response = TgBot::send_api_method("getFile", &CONFIG.telegram_bot_token, [("file_id",  telegram_fileid.as_str().unwrap())]).await.unwrap();
    let response_json = response;
    let telegram_filepath = &response_json["result"]["file_path"];
    println!("{}", telegram_filepath);

    let response = reqwest::get(format!("https://api.telegram.org/file/{}/{}", CONFIG.telegram_bot_token, telegram_filepath.as_str().unwrap())).await.unwrap();
    let response_image = response.bytes().await.unwrap();

    let save_path = telegram_filepath.as_str().unwrap().split("/").last().unwrap();
    let save_path = Path::new(save_path).extension().unwrap();
    println!("{:?}", save_path);

    let name_file = format!("{}.{}", uuid::Uuid::new_v4(), save_path.to_str().unwrap());

    std::fs::write(format!("{}{}",CONFIG.path_to_save_img, &name_file), response_image).unwrap();

    name_file
}

async fn check_callback_query(
    cache: &State<crate::user_account::CacheSessions>,
    callback: &Value)
{

    if let Some(message) = callback.get("message") {

        let message_id = message["message_id"].as_i64().unwrap();
        let chat_id =message["chat"]["id"].as_i64().unwrap();

        TgBot::send_api_method("deleteMessage", &CONFIG.telegram_bot_token, &[
            ("message_id", message_id.to_string()),
            ("chat_id", chat_id.to_string()),
            ("disable_notification", true.to_string())
        ]).await.unwrap();
    }

    if let Some(callback_data) = callback.get("data") {

        let callback_data_str = callback_data.as_str().unwrap();
        let vec_data = callback_data_str.split(":").collect::<Vec<&str>>();
        let (state_auth, token) = (vec_data[0], vec_data[1]);

        if let Some(session) = cache.remove_session(token).await {

            match (state_auth, session) {
                ("ConfirmedLogin", StateAuthUser::WaitConfirm(user)) => {
                    cache.insert_session(
                        token.to_string(),
                        StateAuthUser::LoginConfirm(user)
                    ).await;
                    return;
                },

                ("FailureLogin", StateAuthUser::WaitConfirm(_)) => {
                    cache.insert_session(
                        token.to_string(),
                        StateAuthUser::LoginFailure(
                            "Пользователь отклонил подтверждение входа".to_string()
                        )
                    ).await;
                    return;
                },

                _ => {},
            }
        }

        println!("Токена {} не было в кеше", token);
        return;
    }
}

pub fn state() -> AdHoc {
    AdHoc::on_ignite(
        "telegram_bot",
        |rocket| async {
            rocket.mount(format!("/telegram_bot/{}/", CONFIG.telegram_bot_token),
                         routes![get_tg_update])
        }
    )
}