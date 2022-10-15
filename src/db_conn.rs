
//use crate::sqlite_conn::{document::Document};
use rocket_sync_db_pools::rusqlite::{self, params};
use std::{
    path,
    fmt::{self},
    path::PathBuf,
    str::FromStr,
    collections::HashMap
};
use rocket::{
    data::{
        self,
        FromData,
        ToByteUnit
    },
    Data,
    Request,
    request,
    serde::{
        Serialize,
        Deserialize,
        json::Json
    },
    http::{ContentType, Status}
};

//use crate::sqlite_conn::user::User;

#[derive(Debug)]
pub enum Error {
    TooLarge,
    NoColon,
    InvalidAge,
    Io(std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Document {
    pub title: String,
    pub path: String,
    pub author: User,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>
}

/*
#[rocket::async_trait]
impl<'r> FromData<'r> for Document {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use Error::*;
        use rocket::outcome::Outcome::*;

        let doc_ct = ContentType::new("application", "form-data");
        if req.content_type() != Some(&doc_ct) {
            return Forward(data);
        }

        let limit = req.limits().get("document").unwrap_or(512.bytes());

        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, TooLarge)),
            Err(e) => return Failure((Status::InternalServerError, Io(e))),
        };

        let mut string: String = dbg!(request::local_cache!(req, string).to_string());

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
                avatar: "None".to_string(),
                role: "None".to_string(),
                admin: "None".to_string(),
                tg_id: 0,
                uuid: v[2].clone()
            },
            subject: v[3].clone(),
            type_work: v[4].clone(),
            number_work: 0,//match v[5].parse() {
                //Ok(number_work) => number_work,
                //Err(_) => return Failure((Status::UnprocessableEntity, InvalidAge)),
            //},
            note: Some(v[6].clone())
        })
    }
}

 */

pub fn get_all_user(conn: &mut rusqlite::Connection) -> Vec<User> {
    let mut stmt = conn.prepare("SELECT * FROM users").unwrap();

    stmt
        .query_map([], |row| {
            Ok(
                User{
                    name: row.get(0).unwrap(),
                    nickname: row.get(1).unwrap(),
                    avatar: row.get(2).unwrap(),
                    role: row.get(3).unwrap(),
                    admin: row.get(4).unwrap(),
                    tg_id: row.get(5).unwrap(),
                    uuid: row.get(6).unwrap()
                }
            )
        })
        .unwrap()
        .map(|res|{res.unwrap()})
        .collect()
}


pub fn get_user(conn: &rusqlite::Connection, dict: HashMap<String, String>) -> Option<Vec<User>>{
    let mut execute_str = format!("SELECT * FROM users WHERE ");
    for (column, value) in dict {
        let vec_ch = value.chars();
        let mut new_value_str = String::new();
        for v in vec_ch { //Отслеживание SQl инъекций
            match v {
                '\'' => {continue}//println!("Одинар"),
                '\"' => {continue}//println!("Двойная"),
                _ => new_value_str.push(v)
            }
        }
        let tmp = format!("{} = '{}' AND ", column, new_value_str);
        execute_str += &tmp;
    }
    let res = &execute_str[0..execute_str.len()-4]; // Послоедние слова всегдла будут AND

    let mut stmt = conn.prepare(res).unwrap();
    let users_vec = stmt.query_map([], |row| {
        Ok(
            User{
                name: row.get(0).unwrap(),
                nickname: row.get(1).unwrap(),
                avatar: row.get(2).unwrap(),
                role: row.get(3).unwrap(),
                admin: row.get(4).unwrap(),
                tg_id: row.get(5).unwrap(),
                uuid: row.get(6).unwrap()
            }
        )
    })
        .unwrap()
        .map(|res_user| {res_user.unwrap()})
        .collect();

    Some(users_vec)
}


pub fn get_doc(dict: (HashMap<String, String>, Option<HashMap<String, String>>), conn: &rusqlite::Connection) -> Vec<Document>{

    let mut sql_execute_str = String::from("SELECT * FROM users, documents WHERE (documents.author_uuid = users.uuid) ");

    fn check_inject_sql(st: String) -> String {
        let mut res = String::new();
        for (i, val) in st.chars().enumerate() { //Отслеживание SQl инъекций
            if !val.is_ascii_punctuation(){
                //println!("{}", val);
                //res.push(val)
                res.push(val);
            }
        }
        res
    }

    for (key, val) in dict.0 {
        let tmp = check_inject_sql(val);
        sql_execute_str += &format!(r##"AND documents.{key} = '{tmp}'"##);

    }
    if let Some(users_args) = dict.1 {
        for (key, val) in users_args {
            let tmp = check_inject_sql(val);
            sql_execute_str += &format!(r##"AND users.{key} = '{tmp}'"##);
        }
    }

    let mut stmt = conn.prepare(&dbg!(sql_execute_str)).unwrap();
    let docs_vec = stmt.query_map([], |row| {
        Ok(
            Document{
                title: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
                author:
                User {
                    name: row.get(7).unwrap(),
                    nickname: row.get(8).unwrap(),
                    avatar: row.get(9).unwrap(),
                    role: row.get(10).unwrap(),
                    admin: row.get(11).unwrap(),
                    tg_id: row.get(12).unwrap(),
                    uuid: row.get(13).unwrap()
                },
                subject: row.get(3).unwrap(),
                type_work: row.get(4).unwrap(),
                number_work: row.get(5).unwrap(),
                note: row.get(6).unwrap()
            }
        )
    })
        .unwrap()
        .map(|res_user| {res_user.unwrap()})
        .collect::<Vec<Document>>();

    return docs_vec
}
//
// fn get_user_by_uuid(uuid: String, conn: &rusqlite::Connection) -> Option<User>{
//     let vec_ch = uuid.chars();
//     let mut new_value_uuid = String::new();
//     for v in vec_ch { //Отслеживание SQl инъекций
//         match v {
//             '\'' => {continue}//println!("Одинар"),
//             '\"' => {continue}//println!("Двойная"),
//             _ => new_value_uuid.push(v)
//         }
//     }
//     let sql_execute = format!("SELECT * from users WHERE uuid = '{}'", new_value_uuid);
//     let mut stmt = conn.prepare(&sql_execute).unwrap();
//     let mut users_vec = stmt.query_map([], |row| {
//         Ok(
//             User{
//                 name: row.get(0).unwrap(),
//                 nickname: row.get(1).unwrap(),
//                 avatar: row.get(2).unwrap(),
//                 role: row.get(3).unwrap(),
//                 admin: row.get(4).unwrap(),
//                 tg_id: row.get(5).unwrap(),
//                 uuid: row.get(6).unwrap()
//             }
//         )
//     })
//         .unwrap()
//         .map(|res_user| {res_user.unwrap()})
//         .collect::<Vec<User>>();
//
//     users_vec.pop()
// }


pub fn get_all_users_uuid(conn: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT uuid from users").unwrap();
    stmt
        .query_map([], |row| Ok(row.get(0).unwrap()))
        .unwrap()
        .map(|res| {res.unwrap()})
        .collect::<Vec<String>>()
}

pub fn add_doc(conn: &rusqlite::Connection, doc: Json<Document>) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO documents VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            doc.title,
            doc.path,
            doc.author.uuid,
            doc.subject,
            doc.type_work,
            doc.number_work,
            doc.note
        ]
    )
}