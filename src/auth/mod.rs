use std::borrow::BorrowMut;
use rocket::{
    Rocket, Build, Request,
    Response, execute, fairing::AdHoc,
    form::Form, http::CookieJar
};
use rocket_sync_db_pools::rusqlite::{
    self,
    Connection,
    OptionalExtension,
    params
};
use reqwest;

use crate::telegram_bot::TgBot;
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
        let keyboard = TgBot::get_login_confirmation_keyboard(&token_session);
        TgBot::send_message(&[
            ("chat_id", tg_id_user.to_string().as_str()),
            ("text", "Подтвержаете вход?"),
            ("reply_markup", keyboard.as_str())
        ]).await;
        return "Подтвердите вход";
    }
    return "Пользователь не зарегестрирован";
}

fn create_user_token() {

}

#[get("/tt")]
async fn test_cock(cookies: &CookieJar<'_>) -> Option<String> {
   // cookies.add(rocket::http::Cookie::new("message", "hello!"));
    cookies.get("message").map(|crumb| format!("Message: {}", crumb.value()))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite(
        "Auth stage",
        |rocket| async {
            rocket.mount("/auth", routes![auth])
        }
    )
}