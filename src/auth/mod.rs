use std::{
    collections::HashMap,
    sync::Mutex
};
use rocket::{
    serde::{json::Json, Serialize},
    fairing::AdHoc,
    http::CookieJar,
    State
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
use crate::{Db, api::user::User};

#[derive(Serialize, Debug)]
struct AuthUser {
    result: bool,
    user: Option<User>,
    error: Option<String>,
}

// Использовать асинхронный Mutex
#[derive(Serialize)]
pub struct CacheTokens(
    pub Mutex<HashMap<String, (bool, User)>>
);

#[get("/get_all_session")]
async fn all_session(state: &State<CacheTokens>) -> Json<&CacheTokens> {
    Json(state.inner())
}

#[get("/?<nickname>&<new_session>")]
async fn auth<'a>(
    state: &State<CacheTokens>,
    db: Db,
    cookies: &CookieJar<'a>,
    nickname: &'a str,
    new_session: bool,
) -> Json<AuthUser> {
    use uuid::Uuid;

    if new_session {
        cookies.remove(rocket::http::Cookie::named("session_token"));
    }

    if let Some(token_cookie) = cookies.get("session_token") {
        let token_val = token_cookie.value().to_owned();

        // Если токен есть в кеше.
        if let Ok(mutex_hm) = state.inner().0.try_lock() {
            if let Some(token_status) = mutex_hm.get(&token_val) {
                return
                    // Добавить проверку на изменение запрашиваемого ника.
                    match token_status {
                        (true, user) => { Json(AuthUser{result: true, user: Some(user.clone()), error: None })},
                        (false, _) => { Json(AuthUser{result: false, user: None, error: Some("Waiting auth".to_string()) }) }
                    }
            }
            cookies.remove(rocket::http::Cookie::named("session_token"));
            return Json(AuthUser{result: false, user: None, error: Some("Не смогли получить ваш токен сессии из кеша. Кеш был очищен.".to_string()) })
        }
        return Json(AuthUser{result: false, user: None, error: Some("Не можем проверить подлинность токена (Мьютекс не работает)".to_string()) })
    }

    let nickname = nickname.to_string();
    let nick = nickname.clone();

    let user_opt: Option<User> = db.run(move |conn: &mut Connection| {
        conn.query_row(
            "SELECT * FROM users WHERE nickname = ?1",
            [nick],
            |row| {
                Ok(
                    User{
                        name: row.get_unwrap(0),
                        nickname: row.get_unwrap(1),
                        avatar: row.get_unwrap(2),
                        role: row.get_unwrap(3),
                        admin: row.get_unwrap(4),
                        tg_id: row.get_unwrap(5),
                        uuid: row.get_unwrap(6)
                    }
                )
            }
        ).optional().expect("Ошибка при поиске пользователя по никнейму(Ошибка БД)")
    }).await;

    if let Some(user) = user_opt {
        let user_tg_id = user.tg_id;
        let user_nickname = user.nickname.clone();
        // Генерирование токена
        let token_session = Uuid::new_v4().to_string();

        if let Ok(mut mutex) = state.inner().0.try_lock() {
            mutex.insert(token_session.clone(), (false, user));
        } else {
            return Json(AuthUser{result: false, user: None, error: Some("False added token in cache".to_string()) })
        }

        let conf_login_with_token = format!("ConfirmedLogin:{}", &token_session);
        let fail_login_with_token = format!("FailureLogin:{}", &token_session);
        TgBot::send_message(&BOT_TOKEN, &[
            ("chat_id", user_tg_id.to_string()),
            ("text", format!("Подтвержаете вход? С кодом входа: {}", token_session.split("-").last().unwrap())),
            ("reply_markup", create_login_keyboard(&conf_login_with_token, &fail_login_with_token))
        ]).await;

        cookies.add(rocket::http::Cookie::new(
            "session_token",
            token_session.clone())
        );

        return Json(AuthUser{result: false, user: None, error: Some(format!("Подтвердите вход. Код входа: {}", token_session.split("-").last().unwrap())) })
    }
    return Json(AuthUser{result: false, user: None, error: Some("Пользователь не зарегестрирован. Регистрация просиходит через telegram bota".to_string()) })
}

fn create_login_keyboard(conf_login_with_token: &str, fail_login_with_token: &str) -> String {
    let mut button_accept = HashMap::new();
    button_accept.insert("text", "Yes");
    button_accept.insert("callback_data", conf_login_with_token);

    let mut button_denial = HashMap::new();
    button_denial.insert("text", "No");
    button_denial.insert("callback_data", fail_login_with_token);

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
                .mount("/auth", routes![auth, all_session])
                .manage(
                    CacheTokens(
                        Mutex::new(HashMap::<String, (bool, User)>::new())
                    )
                )
        }
    )
}