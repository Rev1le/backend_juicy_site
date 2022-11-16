use crate::api::PATH_FOR_SAVE_DOCS;
use super::document::{Document, DocumentFromRequest};
use super::user::User;

use rocket_sync_db_pools::rusqlite::{self, params};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write
};
use uuid::Uuid;

use rocket::{
    serde::{
        Serialize,
        Deserialize,
        json::Json
    },
    data::DataStream,
    form::Form
};

// изменить path на type_doc ИБО имя файла - uuid_docБ
//   а путь к хранилищу файлов может быть динамическим

fn check_inject_sql(st: String) -> String {
    let mut res = String::new();

    for val in st.chars() { //Отслеживание SQl инъекций
        if !val.is_ascii_punctuation() { //Если символ не пунктуация
            res.push(val);
        }
    }
    res
}

pub fn get_all_user(
    conn: &mut rusqlite::Connection
) -> Vec<User> {
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
        .map(|res| res.unwrap())
        .collect()
}

pub fn get_user(
    conn: &rusqlite::Connection,
    dict: HashMap<String, String>
) -> Option<Vec<User>>{

    let mut execute_str = String::from("SELECT * FROM users WHERE ");

    for (column, value) in dict {
        let tmp = format!(
            "{} = '{}' AND ",
            column,
            check_inject_sql(value)
        );
        execute_str += &tmp;
    }
    // Послоедние слова всегдла будут AND
    let res = &execute_str[0..execute_str.len()-4];

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


pub fn get_doc(
    dict: (HashMap<String, String>, Option<HashMap<String, String>>),
    conn: &rusqlite::Connection
) -> Vec<Document> {

    let mut sql_execute_str =
        String::from(
            "SELECT * FROM users, documents WHERE (documents.author_uuid = users.uuid) ");

    for (key, val) in dict.0 {
        //let tmp = val;
        sql_execute_str += &format!(r##"AND documents.{} = '{}'"##, key, val);

    }
    // Если был запрошен автор документа
    if let Some(users_args) = dict.1 {
        for (key, val) in users_args {
            //let tmp = check_inject_sql(val);
            sql_execute_str += &format!(r##"AND users.{} = '{}'"##, key, val);
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
        .map(
            |res_user| res_user.unwrap()
        )
        .collect::<Vec<Document>>();

    return docs_vec
}

pub fn get_all_users_uuid(conn: &rusqlite::Connection) -> Vec<String> {
    let mut stmt = conn.prepare("SELECT uuid from users").unwrap();
    stmt
        .query_map(
            [],
            |row| Ok(row.get(0).unwrap())
        )
        .unwrap()
        .map(
            |res| res.unwrap()
        )
        .collect::<Vec<String>>()
}

pub fn add_doc(
    conn: &rusqlite::Connection,
    doc: Document
) -> bool {
    let tmp = conn.execute(
        "INSERT INTO documents VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            doc.title,
            doc.path,
            doc.author.uuid,
            check_inject_sql(doc.subject),
            check_inject_sql(doc.type_work),
            doc.number_work,
            if doc.note == None {
                "None".to_string()
            } else {
                doc.note.unwrap().to_string()
            },
             doc.doc_uuid.unwrap().to_string()
        ]
        );
    println!("{:?}", &tmp);
    return tmp.is_ok()
}

pub fn del_doc(conn: &rusqlite::Connection, doc_uuid: &str) -> bool {

    match Uuid::parse_str(&doc_uuid) {
        Ok(_) => {
            let mut tmp = HashMap::new();
            tmp.insert("doc_uuid".to_string(), doc_uuid.to_string());
            let path_file_doc = get_doc(
                (tmp, None),
                conn
            )
                .get(0)
                .unwrap()
                .path
                .clone();
            fs::remove_file(&path_file_doc).unwrap();
            conn.execute(
                "DELETE FROM documents WHERE doc_uuid = (?1)",
                [doc_uuid]
            ).is_ok()
        },
        Err(_) => false
    }
}

pub fn update_doc(
    conn: &rusqlite::Connection,
    hm: HashMap<String, String>,
    doc_uuid: String
) -> bool {

    let mut sql_execute_str = String::from("UPDATE documents SET ");

    for (key, val) in hm{
        sql_execute_str += &format!(
            r##"{} = '{}', "##,
            key,
            val
        );
    }
    // Удаляем пробел и запятую после последнего элемента
    sql_execute_str.pop();
    sql_execute_str.pop();

    sql_execute_str += &format!("WHERE doc_uuid = '{}'", doc_uuid);
    println!("{}", &sql_execute_str);
    conn
        .execute(&sql_execute_str, [])
        .is_ok()
}