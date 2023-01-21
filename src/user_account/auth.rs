use std::{
    collections::HashMap,
};
use rocket::{
    serde::{json::Json, Serialize, Deserialize},
    fairing::AdHoc,
    http::{CookieJar, Cookie},
    State
};

use rocket_sync_db_pools::rusqlite::Connection;
use TgBot_api::TelegramBotMethods;

use crate::api::user::User;
use crate::db_conn;

use crate::telegram_bot::{TgBot, types};
use crate::{CONFIG, Db};
use crate::user_account::{CacheSessions, StateAuthUser};

#[derive(Clone, Serialize, Debug)]
enum AuthUserResult {
    Ok(User),
    Wait(String),
    Error(String),
}


#[get("/get_all_session")]
async fn all_session(cache: &State<CacheSessions>) -> Json<HashMap<String, StateAuthUser>> {
    Json(cache.get_all_sessions().await)
}

#[get("/?<nickname>")]
async fn auth<'a>(
    cache: &State<CacheSessions>,
    db: Db,
    cookies: &CookieJar<'a>,
    nickname: String,
) -> Json<AuthUserResult> {

    // Доделать чтобы создавалась сразу мновая сессия

    match cookies.get("session_token") {
        // Если токен записан в куки клиента
        Some(cookie_token) => get_session(cookie_token.value(), cache, cookies).await,

        // Если токена нет в куки
        None => create_session(nickname, db, cache, cookies).await
    }
}

async fn get_session(
    token: &str,
    cache: &CacheSessions,
    cookies: &CookieJar<'_>
) -> Json<AuthUserResult> {

    // Если сессия с данным токеном найдена в кеше
    if let Some(state_session) = cache.get_session(&token).await {

        return match state_session {

            StateAuthUser::LoginConfirm(user) => Json(AuthUserResult::Ok(user)),

            StateAuthUser::WaitConfirm(_) => Json(AuthUserResult::Wait(
                format!("Ожидание подтверждения сессии:{}", token.split("-").last().unwrap())
            )),

            StateAuthUser::LoginFailure(_error) => Json(AuthUserResult::Error("Вход был проигнорирован".to_string())),
        }
    }

    cookies.remove(Cookie::named("session_token"));

    return Json(
        AuthUserResult::Error("Удаление старого токена.. перезагрузите страницу".to_string())
    )
}

async fn create_session(nickname: String, db: Db, cache: &CacheSessions, cookies: &CookieJar<'_>) -> Json<AuthUserResult> {
    use uuid::Uuid;

    let user_opt: Option<User> = db.run(
        move |conn: &mut Connection| {
            db_conn::get_user_by_nickname(conn, &nickname)
        }
    ).await;

    if let Some(user) = user_opt {

        let user_tg_id = user.tg_id;
        // Генерирование токена
        let token_session = Uuid::new_v4().to_string();
        let small_code;

        // отправляем в телеграм подтверждение входа
        {
            let conf_login_with_token = format!("ConfirmedLogin:{}", &token_session);
            let fail_login_with_token = format!("FailureLogin:{}", &token_session);
            small_code = token_session.as_str().split("-").last().unwrap();
            let text_message = format!("Подтвержаете вход? С кодом входа: {}", small_code);

            TgBot::send_api_method("sendMessage", &CONFIG.telegram_bot_token, &[
                ("chat_id", user_tg_id.to_string()),
                ("text", text_message),
                ("reply_markup", create_login_keyboard(&conf_login_with_token, &fail_login_with_token))
            ]).await.unwrap();
        }

        // Добавляем сессию в кеш
        cache.insert_session(
            token_session.clone(),
            StateAuthUser::WaitConfirm(user)
        ).await;

        // Добавляем токен в куки
        cookies.add(Cookie::new(
            "session_token",
            token_session.clone())
        );

        return Json(
            AuthUserResult::Wait(small_code.to_string())
        );

    }
    // Пользователя нет в БД
    return Json(
        AuthUserResult::Error("Пользователь не зарегестрирован".to_string())
    );
}

fn create_login_keyboard(conf_login_with_token: &str, fail_login_with_token: &str) -> String {

    let button_accept = types::inline_keyboard::InlineKeyboardButton::new(
        "Yes".to_string(),
        None, Some(conf_login_with_token.to_string()),
        None, None,
        None, None);

    let button_denied = types::inline_keyboard::InlineKeyboardButton::new(
        "No".to_string(),
        None, Some(fail_login_with_token.to_string()),
        None, None,
        None, None);

    let keyboard = types::inline_keyboard::InlineKeyboardMarkup {
        inline_keyboard: vec![vec![button_accept], vec![button_denied]]
    };

    keyboard.keyboard_as_str()
}

#[get("/quit")]
async fn exit_acc(cache: &State<CacheSessions>, cookies: &CookieJar<'_>) -> Json<bool> {
    if let Some(cookie_token) = cookies.get("session_token") {
        cache.remove_session(cookie_token.value()).await;
        return Json(true)
    }

    Json(false)
}

pub fn state() -> AdHoc {
    AdHoc::on_ignite(
        "Auth stage",
        |rocket| async {
            rocket
                .mount("/auth", routes![auth])
        }
    )
}