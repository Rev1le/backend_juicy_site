use std::collections::HashMap;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub name: String,
    pub nickname: String,
    pub avatar: String,
    pub role: String,
    pub admin: String,
    pub tg_id: i64,
    pub uuid: String,
}

//Структура для запроса пользователя
#[derive(Debug, FromForm, Copy, Clone)]
pub struct UserFromRequest<'a> {
    name: Option<&'a str>,
    nickname: Option<&'a str>,
    avatar: Option<&'a str>,
    role: Option<&'a str>,
    admin: Option<&'a str>,
    tg_id: Option<&'a str>,
    uuid: Option<&'a str>,
}

impl<'a> UserFromRequest<'a> {

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut hm = HashMap::new();

        if let Some(name) = self.name {
            hm.insert(
                "name".to_string(),
                name.to_string()
            );
        }
        if let Some(nickname) = self.nickname {
            hm.insert(
                "nickname".to_string(),
                nickname.to_string()
            );
        }
        if let Some(avatar) = self.avatar {
            hm.insert(
                "avatar".to_string(),
                avatar.to_string()
            );
        }
        if let Some(role) = self.role {
            hm.insert(
                "role".to_string(),
                role.to_string()
            );
        }
        if let Some(admin) = self.admin {
            hm.insert(
                "admin".to_string(),
                admin.to_string()
            );
        }
        if let Some(tg_id) = self.tg_id {
            hm.insert(
                "tg_id".to_string(),
                tg_id.to_string()
            );
        }
        if let Some(uuid) = self.uuid {
            hm.insert(
                "uuid".to_string(),
                uuid.to_string()
            );
        }
        hm
    }
}