/*
use std::fmt;
use rocket::{data::{
    self,
    FromData
}, Data, Request, request, serde::{
    Serialize,
    Deserialize
}};
use rocket::data::ToByteUnit;
use rocket::error::ErrorKind::Io;
use rocket::http::{ContentType, Status};

use crate::sqlite_conn::user::User;

#[derive(Debug)]
enum Error {
    TooLarge,
    NoColon,
    InvalidAge,
    Io(std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub title: String,
    pub path: String,
    pub author: User,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for Document {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use Error::*;
        use rocket::outcome::Outcome::*;

        let doc_ct = ContentType::new("application", "x-doc");
        if req.content_type() != Some(&doc_ct) {
            return Forward(data);
        }

        let limit = req.limits().get("document").unwrap_or(512.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, TooLarge)),
            Err(e) => return Failure((Status::InternalServerError, Io(e))),
        };

        let mut string: String = request::local_cache!(req, string).to_string();

        //let (title, path, author, subject, type_work, number_work, note) = match string.find(':') {
        //    Some(i) => (string[..i], string[(i + 1)..]),
        //    None => return Failure((Status::UnprocessableEntity, NoColon)),
        //};

        let mut v = Vec::<String>::with_capacity(7);

        while string.find(':') != None {
            if let Some(i) = string.find(':') {
                v.push(string[..i].to_string());
                string.remove(i);
            }
        }

        if v.len() < 7 {
            return Failure((Status::UnprocessableEntity, NoColon))
        }

        Success(Document {
            title: v[0].clone(),
            path: v[1].clone(),
            author: User {
                name: "None".to_string(),
                nickname: "None".to_string(),
                avatar: Default::default(),
                role: "None".to_string(),
                admin: false,
                tg_id: 0,
                uuid: v[2].clone()
            },
            subject: v[3].clone(),
            type_work: v[4].clone(),
            number_work: match v[5].parse() {
                Ok(number_work) => number_work,
                Err(_) => return Failure((Status::UnprocessableEntity, InvalidAge)),
            },
            note: Some(v[6].clone())
        })
    }
}

impl Document {
    pub fn new(
        title: String,
        path: String,
        author: User,
        subject: String,
        type_work: String,
        number_work: i64,
        note: Option<String>,
    ) -> Document {
        Document {
            title,
            path,
            author,
            subject,
            type_work,
            number_work,
            note,
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "||Сведения об документе||\nДокумент: {}\nРасположение: {}\nАвтор: {}\nПредмет: {}\n\
            Тип работы: {}\nНомер работы: {}\nПримечание: {:?}\n",
            self.title, self.path, self.author, self.subject, self.type_work,
            self.number_work, self.note
        )
    }
    /*
    fn vector_to_struct(vec: &Vec<String>) -> Self {
        Document::new_user(
            vec[0].clone(),
            PathBuf::from(vec[2].clone()),
            vec[0].clone(),
            vec[3].clone(),
            FromStr::from_str(vec[4].clone().as_str()).unwrap(),
            FromStr::from_str(vec[5].clone().as_str()).unwrap(),
            vec[6].clone()
        )
    }

     */
}

 */