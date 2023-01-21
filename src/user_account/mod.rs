// Авторизация на сайте
mod auth;

use std::collections::HashMap;
use rocket::{
    State,
    serde::{Serialize, Deserialize, json::Json},
    fairing::AdHoc,
    http::{Cookie, CookieJar},
    tokio::sync::Mutex
};
use crate::api::user::User;

#[derive(Serialize, Deserialize, Clone)]
pub enum StateAuthUser {
    LoginConfirm(User),
    WaitConfirm(User),
    LoginFailure(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataAccess<T, E> {
    Allowed(T),
    Denied(E),
}

// Таблица ключ -токен, значение - состояние учетной записи
pub struct CacheSessions(Mutex<HashMap<String, StateAuthUser>>);

impl CacheSessions {

    pub fn new() -> Self {
        CacheSessions(Mutex::new(HashMap::default()))
    }

    pub async fn get_all_sessions(&self) -> HashMap<String, StateAuthUser> {
        self.0.lock().await.clone()
    }

    pub async fn get_session(&self, token: &str) -> Option<StateAuthUser> {
        self.0.lock().await.get(token).cloned()
    }

    pub async fn get_user_authconfirm(&self, token: &str) -> Option<User> {
         if let Some(StateAuthUser::LoginConfirm(user)) = self.get_session(token).await {
             return Some(user.clone())
         }

        None
    }

    pub async fn insert_session(&self, token: String, session: StateAuthUser) -> Option<StateAuthUser> {
        self.0.lock().await.insert(token, session)
    }

    pub async fn remove_session(&self, token: &str) -> Option<StateAuthUser> {
        self.0.lock().await.remove(token)
    }
}

// Сипользуется для отладки
#[get("/?<ind>")]
async fn add_session(cache: &State<CacheSessions>, cookie: &CookieJar<'_>, ind: i64) {
    use crate::telegram_bot::get_user_avatar;
    use uuid::Uuid;
    let token = Uuid::new_v4().to_string();

    let temp_user =  User {
        name: "Roman".to_string(),
        nickname: "Rev1le".to_string(),
        avatar: String::new(),
        role: String::new(),
        admin: String::new(),
        tg_id: i64::default(),
        uuid: String::new(),
    };

    match ind {
        1 => {
            cache.insert_session(
                token.clone(),
                StateAuthUser::LoginConfirm(temp_user)
            ).await;
        },
        2 => {
            cache.insert_session(
                token.clone(),
                StateAuthUser::WaitConfirm(temp_user)
            ).await;
        },
        3 => {
            cache.insert_session(token.clone(), StateAuthUser::LoginFailure(
                "Error dada".to_string()
            )).await;
        },
        _ => {}
    }

    get_user_avatar(490481406).await;

    cookie.add(Cookie::new("session_token", token.clone()))
}

// Информация об сессии пользовтаеля
#[get("/info")]
async fn get_acc_data(cache: &State<CacheSessions>, cookie: &CookieJar<'_>) -> Json<Option<StateAuthUser>> {

    if let Some(cookie_token) = cookie.get("session_token") {
        let token = cookie_token.value();

        if let Some(session) = cache.get_session(token).await {
            return Json(Some(session))
        }
    }

    Json(None)
}

pub fn state() -> AdHoc {
    AdHoc::on_ignite(
        "Account State",
        |rocket| async {
            rocket
                .mount("/session", routes![get_acc_data, add_session])
                .attach(auth::state())
                .manage(CacheSessions(Mutex::new(HashMap::default())))
        }
    )
}