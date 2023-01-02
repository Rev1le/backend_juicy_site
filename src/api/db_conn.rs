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
    param: HashMap<String, String>
) -> Result<Vec<User>, rusqlite::Error>{

    let mut execute_str = String::from("SELECT * FROM users WHERE ");

    for (column, value) in param {
        execute_str.push_str(&format!("{} = '{}' AND ", column, value));
    }
    // Послоедние слова всегда будут AND
    let execute_str = &execute_str[0..execute_str.len()-4];

    let mut stmt = conn.prepare(execute_str)?;

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
        })?
        .map(|res_user| res_user.unwrap())
        .collect::<Vec<User>>();

    Ok(users_vec)
}


pub fn get_doc(
    doc: HashMap<String, String>,
    opt_user: Option<HashMap<String, String>>,
    conn: &rusqlite::Connection
) -> Result<Vec<Document>, rusqlite::Error> {

    //let dict: (HashMap<String, String>, HashMap<String, String>) = (doc, user);

    let mut sql_execute_str =
        String::from(
            "SELECT * FROM users, documents WHERE (documents.author_uuid = users.uuid) ");

    for (key, val) in doc {
        //let tmp = val;
        sql_execute_str += &format!(r##"AND documents.{} = '{}'"##, key, val);
    }

    // Если был запрошен автор документа
    if let Some(user) = opt_user {
        for (key, val) in user {
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
    println!("ddd{:?}", &docs_vec);
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
) -> Result<usize, rusqlite::Error> {
    conn.execute(
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
        ])
}

//путь для сохранения файлов брать из конфига
pub fn del_doc(conn: &rusqlite::Connection, path_for_save_docs: &str, doc_uuid: &str) -> bool {

    let mut tmp = HashMap::default();
    tmp.insert("doc_uuid".to_string(), doc_uuid.to_owned());

    let res_vec_docs = get_doc(tmp, None, conn);
    // Может не вернуть документов, если uuid пользователя не ущетсвует

    match res_vec_docs {
        Ok(vec_docs) => {

            if vec_docs.len() > 0 {
                let mut path = path_for_save_docs.to_string();
                path.push_str(&vec_docs.first().unwrap().path);

                if fs::remove_file(&path).is_ok() {
                    return
                        conn.execute(
                            "DELETE FROM documents WHERE doc_uuid = (?1)",
                            [doc_uuid]
                        ).is_ok()
                } else {
                    println!("Файл не был удален файловой системой");
                }
            } else {
                println!("Документа по заданному uuid не был найден");
            }
        },
        Err(_) => {}
    }
    return false
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