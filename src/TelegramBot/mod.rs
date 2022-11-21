use std::fmt::Debug;
use std::fs::read_to_string;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::serde::json::Value;
use rocket_sync_db_pools::rusqlite;
use rocket_sync_db_pools::rusqlite::{Connection, OptionalExtension};
use crate::Db;

use uuid::Uuid;

// struct Update {
//     update_id: isize,
//     message: Message,
//
// }

#[post("/", data="<update_data>")]
async fn get_tg_update<'a>(db: Db, update_data: Json<Value>) -> Json<bool> {
    println!("{:?}", update_data["update_id"]);
    println!("{:?}", update_data["message"]["text"]);

    let upd_id = update_data["update_id"].as_i64().unwrap();

    if let Some(message) = update_data.get("message") {
        check_message(message, db).await;
    }

    if let Some(callback_query) = update_data.get("callback_query") {
        check_callback_query(callback_query);
    }

    Json(true)
}

async fn check_message(message: &Value, db: Db) {
    let user_id = match message.get("from") {
        Some(user) => user["id"].as_i64().unwrap(),
        None => return
    };

    let username = match message.get("from") {
        Some(user) => user["first_name"].as_str().unwrap(),
        None => return
    };
    let chat_id = message["chat"]["id"].as_i64().unwrap();

    let message_text = match message.get("text") {
        Some(text) => text.as_str().unwrap(),
        None => return
    };

    if Some('/') != message_text.chars().next() {
        println!("This is not command!");
        return;
    }

    let words_vec = message_text.split(" ").collect::<Vec<&str>>();

    if words_vec[0] == "/reg" {
        account_register(
            &words_vec[1..], username, user_id, db
        ).await;
    }
}

async fn account_register(command_args: &[&str], username: &str, user_id: i64, db: Db) {

    let nickname = command_args[0].to_string();

    if let Ok(Some(id)) = db.run(
        move |conn: &mut Connection| {
        conn.query_row(
            "SELECT nickname, tg_id FROM users WHERE nickname=(?)",
            [nickname],
            |row| row.get::<usize, i64>(1))
            .optional()
        }
    ).await {
        println!("Пользователь зареган с tg_id = {}", id);
        return;
    }
    println!("Пользователь не зареган.");

    let mut insert_values =[
        username.to_string(),
        command_args[1].to_string(),
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
    println!("Пользователь успешно зарегестрирован!!!");
}

fn check_callback_query(callback: &Value) {

}


pub fn state() -> AdHoc {
    AdHoc::on_ignite(
        "TelegramBot",
        |rocket| async {
            rocket.mount("/telegrmbot/bot5013260088:AAEeM57yLluiO62jFxef5v4LoG4tkLVvUMA/",
                         routes![get_tg_update])
        }
    )
}