use std::{
    collections::HashMap,
    sync::Mutex
};
use rocket::{serde::json::Json, fairing::AdHoc, http::CookieJar, State};
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
use crate::{Db, api::user::User};

pub(crate) struct CacheTokens(
    pub(crate) Mutex<HashMap<String, bool>>
);

#[get("/?<nickname>")]
async fn auth<'a>(
    state: &State<CacheTokens>,
    db: Db,
    cookies: &CookieJar<'a>,
    nickname: &'a str
) -> &'static str {
    use uuid::Uuid;

    if let Some(token_cookie) = cookies.get("session_token") {
        let token_val = token_cookie.value().to_owned();

        if let Ok(mutex) = state.inner().0.try_lock() {
            if let Some(token_status) = mutex.get(&token_val) {

                return
                    match token_status {
                        true => { "Активен" },
                        false => { "НЕ Активен" }
                    }
            }
        }

        // let active_token_opt = db.run(
        //     |conn: &mut Connection| {
        //         conn.query_row(
        //             "SELECT activate from users_sessions WHERE token=?1",
        //             [token_val],
        //             |row| row.get::<usize, String>(0)
        //         ).optional()
        //     }
        // ).await.unwrap();

        // if let Some(active) = active_token_opt {
        //     if active == "true" {
        //         return "Активен"
        //     }
        //     return "НЕ Активен"
        // }
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

        if let Ok(mut mutex) = state.inner().0.try_lock() {
            mutex.insert(token_session.clone(), false);
        } else {
            return "False added token in cache";
        }

        cookies.add(rocket::http::Cookie::new(
            "session_token",
            token_session.clone()
        ));
        let conf_login_with_token = format!("ConfirmedLogin:{}", token_session);
        TgBot::send_message(&BOT_TOKEN, &[
            ("chat_id", tg_id_user.to_string().as_str()),
            ("text", "Подтвержаете вход?"),
            ("reply_markup", create_login_keyboard(&conf_login_with_token).as_str())
        ]).await;
        return "Подтвердите вход";
    }
    return "Пользователь не зарегестрирован";
}

fn create_login_keyboard(conf_login_with_token: &str) -> String {
    let mut button_accept = HashMap::new();
    button_accept.insert("text", "Yes");
    button_accept.insert("callback_data", conf_login_with_token);

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
            rocket
                .mount("/auth", routes![auth])
                .manage(CacheTokens(Mutex::new(HashMap::<String, bool>::new())))
        }
    )
}