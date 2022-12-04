use rocket_sync_db_pools::rusqlite::{self, params};
use std::{
    collections::HashMap,
    fs,
};
use uuid::Uuid;

use super::document::Document;
use super::user::User;

// изменить path на type_doc ИБО имя файла - uuid_docБ
//   а путь к хранилищу файлов может быть динамическим
pub fn get_user(
    conn: &rusqlite::Connection,
    dict: HashMap<String, String>
) -> Result<Option<Vec<User>>, rusqlite::Error>{

    let mut execute_str = String::from("SELECT * FROM users WHERE ");

    for (column, value) in dict {
        let tmp = format!(
            "{} = '{}' AND ",
            column,
            value
        );
        execute_str += &tmp;
    }
    // Послоедние слова всегдла будут AND
    let res = &execute_str[0..execute_str.len()-4];

    let mut stmt = conn.prepare(res)?;

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
        })?.collect::<Vec<Result<User, rusqlite::Error>>>();

    let mut ret_users_vec = Vec::with_capacity(users_vec.iter().count());

    for res_user in users_vec {
        if let Ok(user) = res_user {
            ret_users_vec.push(user);
        } else {
            println!("Пользователь получен с ошибкой");
        }
    }

    Ok(Some(ret_users_vec))
}


pub fn get_doc(
    doc: HashMap<String, String>,
    user: Option<HashMap<String, String>>,
    conn: &rusqlite::Connection
) -> Result<Vec<Document>, rusqlite::Error> {

    let dict: (HashMap<String, String>, Option<HashMap<String, String>>) = (doc, user);

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
                title: row.get(7)?,
                path: row.get(8)?,
                author:
                User {
                    name: row.get(0)?,
                    nickname: row.get(1)?,
                    avatar: row.get(2)?,
                    role: row.get(3)?,
                    admin: row.get(4)?,
                    tg_id: row.get(5)?,
                    uuid: row.get(6)?
                },
                subject: row.get(10)?,
                type_work: row.get(11)?,
                number_work: row.get(12)?,
                note: row.get(13)?,
                doc_uuid: row.get(14)?
            }
        )
    })?.collect::<Vec<Result<Document, rusqlite::Error>>>();

    let mut ret_doc_vec = Vec::with_capacity(docs_vec.iter().count());

    for res_doc in docs_vec {
        if let Ok(doc) = res_doc {
            ret_doc_vec.push(doc);
        } else {
            println!("Документ получен с ошибкой");
        }
    }

    return Ok(ret_doc_vec)
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
) -> Result<bool, rusqlite::Error> {
    let tmp = conn.execute(
        "INSERT INTO documents VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            doc.title,
            doc.path,
            doc.author.uuid,
            doc.subject,
            doc.type_work,
            doc.number_work,
            if let Some(note) = doc.note  {
                note
            } else {
                String::from("None")
            },
            doc.doc_uuid.to_string()
        ]
        );
    println!("{:?}", &tmp);
    return Ok(tmp.is_ok())
}

pub fn del_doc(path_for_save_docs: &str, conn: &rusqlite::Connection, doc_uuid: &str) -> bool {

    match Uuid::parse_str(&doc_uuid) {
        Ok(_) => {
            let mut tmp = HashMap::new();
            tmp.insert("doc_uuid".to_string(), doc_uuid.to_string());
            let res_opt_path_file_doc = get_doc(tmp, None, conn);

            if res_opt_path_file_doc.is_err() {
                println!("Возникла ошибка при удалении");
                return false
            }

            if let Some(doc) = res_opt_path_file_doc.unwrap().get(0) {

                let mut path = path_for_save_docs.to_string();
                path.push_str(&doc.path);

                fs::remove_file(&path).unwrap();

                return
                    conn.execute(
                        "DELETE FROM documents WHERE doc_uuid = (?1)",
                        [doc_uuid]
                    ).is_ok()
            }
            return false;
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