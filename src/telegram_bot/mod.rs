use rocket_sync_db_pools::rusqlite::{Connection, OptionalExtension};
use rocket::{
    serde::json::{Json, Value},
    fairing::AdHoc
};
pub use TgBot_api::{telegram_bot_methods::TelegramBotMethods, InlineKeyboardMarkup};
use uuid::Uuid;
use crate::Db;
use crate::CONFIG;

pub const BOT_TOKEN: &str = CONFIG.telegram_bot_token;
pub const TG_API: &str = "https://api.telegram.org/bot5013260088:AAEeM57yLluiO62jFxef5v4LoG4tkLVvUMA";

pub struct TgBot;
impl TelegramBotMethods for TgBot {}

#[post("/", data="<update_data>")]
async fn get_tg_update<'a>(db: Db, update_data: Json<Value>) -> Json<bool> {

    println!("{:?}", update_data);

    // Если пришло сообщение
    if let Some(message) = update_data.get("message") {
        check_message(message, &db).await;
    }

    // если бот получил callback сообщение
    if let Some(callback_query) = update_data.get("callback_query") {
        check_callback_query(&db, callback_query).await;
    }

    Json(true)
}

async fn check_message(message: &Value, db: &Db) {

    let chat_id = message["chat"]["id"].as_i64().unwrap();
    let user_id =
        match message.get("from") {
            Some(user) => user["id"].as_i64().unwrap(),
            _ => return
        };
    let username =
        match message.get("from") {
            Some(user) => user["first_name"].as_str().unwrap(),
            _ => return
        };
    let message_text =
        match message.get("text") {
            Some(text) => text.as_str().unwrap(),
            _ => return
        };

    // Является ли сообщение командой
    if Some('/') != message_text.chars().next() {
        println!("This is not command!");
        return;
    }

    let words_vec = message_text
        .split(" ")
        .collect::<Vec<&str>>();

    // Команда регистрации пользователя
    if words_vec[0] == "/reg" {
        if words_vec.len() < 2 {
            return;
        }
        let reg_nickname = words_vec[1];
        account_register(username,  reg_nickname, user_id,chat_id, db).await;
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
        TgBot::send_message(BOT_TOKEN, &[
            ("chat_id", chat_id.to_string().as_str()),
            ("text", "Аккаунт с таким ником уже зарегестрирован.")
        ]).await;
        return;

        // argsDict = {
        //     "chat_id": Chat_id,
        //     "message_id": message_id,
        //     "disable_notification" : True
        // }
    }
    println!("Пользователь не зареган.");

    let insert_values =[
        username.to_string(),
        nickname.to_string(),
        "unknown.png".to_string(),
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
    TgBot::send_message(BOT_TOKEN,
                        &[
            ("chat_id", chat_id.to_string().as_str()),
            ("text", "Пользователь успешно зарегестрирован!!!")
        ]
    ).await;
}

async fn check_callback_query(db: &Db, callback: &Value) {
    if let Some(message) = callback.get("message") {
        let message_id = message["message_id"].as_i64().unwrap();
        let chat_id =message["chat"]["id"].as_i64().unwrap();

        TgBot::delete_message(BOT_TOKEN, &[
            ("message_id", message_id.to_string()),
            ("chat_id", chat_id.to_string()),
            ("disable_notification", true.to_string())
        ]).await;
    }


    if let Some(answer) = callback.get("data") {
        if answer.as_str().unwrap() == "FailureLogin" {
            println!("пользователь не подтвердил вход");
            return;
        }

        let tmp = answer.as_str().unwrap().split(":").collect::<Vec<&str>>();

        if tmp[0] == "ConfirmedLogin" {
            println!("АЙди подтверждения {}", tmp[1]);
            let uuid = tmp[1].to_owned();
            db.run(move |conn: &mut Connection| {
                conn.execute(
                    "UPDATE users_sessions SET activate='true' WHERE token= ?1",
                    [uuid]
                )
            }).await.unwrap();
        }
    }
}


pub fn state() -> AdHoc {
    AdHoc::on_ignite(
        "telegram_bot",
        |rocket| async {
            rocket.mount("/telegrmbot/bot5013260088:AAEeM57yLluiO62jFxef5v4LoG4tkLVvUMA/",
                         routes![get_tg_update])
        }
    )
}