use std::collections::HashMap;
use rocket::{
    fairing::AdHoc,
    http::CookieJar
};
use rocket_sync_db_pools::rusqlite::{
    self,
    Connection,
    OptionalExtension,
};

use crate::telegram_bot::{
    TgBot,
    TelegramBotMethods,
    BOT_TOKEN,
    InlineKeyboardMarkup
};
use crate::Db;

#[get("/?<nickname>")]
async fn auth<'a>(
    db: Db,
    cookies: &CookieJar<'a>,
    nickname: &'a str
) -> &'a str {
    use uuid::Uuid;

    if let Some(token_cookie) = cookies.get("session_token") {
        let token_val = token_cookie.value().to_owned();
        let active_token_opt = db.run(
            |conn: &mut Connection| {
                conn.query_row(
                    "SELECT activate from users_sessions WHERE token=?1",
                    [token_val],
                    |row| row.get::<usize, String>(0)
                ).optional()
            }
        ).await.unwrap();

        if let Some(active) = active_token_opt {
            if active == "true" {
                return "Активен"
            }
            return "NOT Активен"
        }
    }

    let nickname = nickname.to_string();
    let nick = nickname.clone();

    let tg_id_user_opt: Option<i64> = db.run(move |conn: &mut Connection| {
        conn.query_row(
            "SELECT tg_id FROM users WHERE nickname = ?1",
            [nick],
            |row| row.get::<usize, i64>(0)
        ).optional().unwrap()
    }).await;

    if let Some(tg_id_user) = tg_id_user_opt {
        let nick = nickname.clone();
        let token_session = Uuid::new_v4().to_string();
        let token_cl = token_session.clone();
        db.run(
            move |conn: &mut Connection|
                conn.execute(
                    "INSERT INTO users_sessions VALUES(?1, ?2, ?3)",
                    rusqlite::params![token_cl, nick, "false"]
                )
        ).await.unwrap();

        cookies.add(rocket::http::Cookie::new(
            "session_token",
            token_session.clone()
        ));
        TgBot::send_message(BOT_TOKEN, &[
            ("chat_id", tg_id_user.to_string().as_str()),
            ("text", "Подтвержаете вход?"),
            ("reply_markup", create_login_keyboard().as_str())
        ]).await;
        return "Подтвердите вход";
    }
    return "Пользователь не зарегестрирован";
}

fn create_login_keyboard() -> String {
    let mut button_accept = HashMap::new();
    button_accept.insert("text", "Yes");
    button_accept.insert("callback_data", "ConfirmedLogin");

    let mut button_denial = HashMap::new();
    button_denial.insert("text", "No");
    button_denial.insert("callback_data", "FailureLogin");

    let keyboard = InlineKeyboardMarkup {
        inline_keyboard: vec![vec![button_accept], vec![button_denial]]
    };

    keyboard.keyboard_as_str()
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite(
        "Auth stage",
        |rocket| async {
            rocket.mount("/auth", routes![auth])
        }
    )
}