//use crate::sqlite_conn::{document::Document};
use rocket_sync_db_pools::rusqlite::{self, params};
use std::{
    collections::HashMap,
    fs::File,
    io::Write
};

use uuid::Uuid;

use crate::PATH_FOR_SAVE_DOCS;

use rocket::{
    serde::{
        Serialize,
        Deserialize,
        json::Json
    }
};

use crate::DocumentFile;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Document {
    pub title: String,
    pub path: String,
    pub author: User,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
    pub doc_uuid: Option<String>,
}

// изменить path на type_doc ИБО имя файла - uuid_docБ а путь к хранилищу файлов может быть динамическим

pub fn get_all_user(conn: &mut rusqlite::Connection) -> Vec<User> {
    let mut stmt = conn
        .prepare("SELECT * FROM users")
        .unwrap();

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
        .map(|res|{
            res.unwrap()
        })
        .collect()
}


pub fn get_user(conn: &rusqlite::Connection, dict: HashMap<String, String>) -> Option<Vec<User>>{
    let mut execute_str = String::from("SELECT * FROM users WHERE ");

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

    let mut stmt = conn
        .prepare(res)
        .unwrap();

    let users_vec = stmt
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
        .map(|res_user| {res_user.unwrap()})
        .collect();
    Some(users_vec)
}


pub fn get_doc(dict: (HashMap<String, String>, Option<HashMap<String, String>>), conn: &rusqlite::Connection) -> Vec<Document>{

    let mut sql_execute_str = String::from("SELECT * FROM users, documents WHERE (documents.author_uuid = users.uuid) ");

    fn check_inject_sql(st: String) -> String {
        let mut res = String::new();
        for val in st.chars() { //Отслеживание SQl инъекций
            if !val.is_ascii_punctuation() {
                //println!("{}", val);
                //res.push(val)
                res.push(val);
            }
        }
        res
    }

    for (key, val) in dict.0 {
        let tmp = check_inject_sql(val);
        sql_execute_str += &format!(r##"AND documents.{} = '{}'"##, key, tmp);

    }
    if let Some(users_args) = dict.1 {
        for (key, val) in users_args {
            let tmp = check_inject_sql(val);
            sql_execute_str += &format!(r##"AND users.{} = '{}'"##, key, tmp);
        }
    }

    let mut stmt = conn.prepare(&dbg!(sql_execute_str)).unwrap();
    let docs_vec = stmt.query_map([], |row| {
        Ok(
            Document{
                title: row.get(7).unwrap(),
                path: row.get(8).unwrap(),
                author:
                User {
                    name: row.get(0).unwrap(),
                    nickname: row.get(1).unwrap(),
                    avatar: row.get(2).unwrap(),
                    role: row.get(3).unwrap(),
                    admin: row.get(4).unwrap(),
                    tg_id: row.get(5).unwrap(),
                    uuid: row.get(6).unwrap()
                },
                subject: row.get(10).unwrap(),
                type_work: row.get(11).unwrap(),
                number_work: row.get(12).unwrap(),
                note: row.get(13).unwrap(),
                doc_uuid: row.get(14).unwrap()
            }
        )
    })
        .unwrap()
        .map(|res_user| {res_user.unwrap()})
        .collect::<Vec<Document>>();

    return docs_vec
}

pub fn get_all_users_uuid(conn: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT uuid from users").unwrap();
    stmt
        .query_map([], |row| Ok(row.get(0).unwrap()))
        .unwrap()
        .map(|res| {res.unwrap()})
        .collect::<Vec<String>>()
}

pub fn add_doc(
    conn: &rusqlite::Connection,
    doc: Json<DocumentFile>) -> bool
{

    let doc_uuid: String = Uuid::new_v4().to_string();
    let tmp = format!(r"{}.{}", doc_uuid, doc.file_type);

    if File::create(String::from(PATH_FOR_SAVE_DOCS) + &tmp)
        .unwrap()
        .write(&doc.file)
        .is_err() {
        return false
    } else {
        conn.execute(
            "INSERT INTO documents VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
            doc.title,
            tmp,
            doc.author_uuid,
            doc.subject,
            doc.type_work,
            doc.number_work,
            if doc.note == None {
                "None".to_string()
            } else {
                doc.note.as_ref().unwrap().to_string()
            },
            doc_uuid
        ]
        )
            .is_ok()
    }
}

pub fn del_doc(conn: &rusqlite::Connection, doc_uuid: &str) -> bool {

    match Uuid::parse_str(doc_uuid) {
        Ok(_) => {
            conn.execute(
                "DELETE FROM documents WHERE doc_uuid = (?1)",
                [doc_uuid]
            ).is_ok()
        },
        Err(E) => false
    }
}