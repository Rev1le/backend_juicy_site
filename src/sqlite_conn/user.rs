use std::path;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use rocket::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    pub name: String,
    pub nickname: String,
    pub avatar: path::PathBuf,
    pub role: String,
    pub admin: bool,
    pub tg_id: i64,
    pub uuid: String,
}

pub trait UserEvent {
    fn new_user(
        name: String,
        nickname: String,
        avatar: path::PathBuf,
        role: String,
        admin: bool,
        tg_id: i64,
        uuid: String,
    ) -> User;
    fn changing_param() -> Result<bool, String>;
    fn add_document() -> Result<bool, String>;
    fn vector_to_struct(vec: &Vec<String>) -> Self;
}
impl UserEvent for User {
    fn new_user(
        name: String,
        nickname: String,
        avatar: path::PathBuf,
        role: String,
        admin: bool,
        tg_id: i64,
        uuid: String,
    ) -> User {
        User {
            name,
            nickname,
            avatar,
            role,
            admin,
            tg_id,
            uuid,
        }
    }
    fn vector_to_struct(vec: &Vec<String>) -> Self {
        User::new_user(
            vec[0].clone(),
            vec[1].clone(),
            PathBuf::from(vec[2].clone()),
            vec[3].clone(),
            FromStr::from_str(vec[4].clone().as_str()).unwrap(),
            FromStr::from_str(vec[5].clone().as_str()).unwrap(),
            vec[6].clone()
        )
    }
    fn changing_param() -> Result<bool, String> {
        Ok(true)
    }
    fn add_document() -> Result<bool, String> {
        Ok(true)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n     ||Сведения об пользователе||\n     Имя: {}\n     Ник: {}\n     Аватар находится: {:?}\n     Роль: {}\n     \
            Админ: {}\n     Телеграм id: {}\n     uuid: {:?}",
            self.name, self.nickname, self.avatar, self.role, self.admin, self.tg_id, self.uuid
        )
    }
}