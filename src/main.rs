extern crate reqwest;
mod sqlite_conn;

use rocket::{
    http::ContentType,
    serde::{Serialize, Deserialize, json::Json},
    fs::{NamedFile}
};
use std::{fs, collections::HashMap, result};
use rocket::data::N;
use rocket::serde::json::serde_json;
use crate::sqlite_conn::user;
use crate::sqlite_conn::document::Document;
use crate::sqlite_conn::DataBase;

const PATH_BD: &str = r"F:\Projects\Rust\juicy_site\DAtaBase\test.db";

#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<NamedFile> {
    NamedFile::open("icon_site.ico").await.ok()
}

#[get("/")]
fn index() -> (ContentType, String) {
    let html = fs::read_to_string("/home/roma/PythonApps/dsBot/db_html.html").unwrap();
    (ContentType::HTML, html)
}

#[get("/")] // Для отображения списка api адресов
async fn all_api() -> Json<Vec<&'static str>> {
    Json(vec!["/get_photo", "/upload_files?<url>", "/get"])
}

#[get("/get_photo?<name>")]
pub async fn get_files(name: Option<&str>) -> Option<NamedFile> {
    // Добавить првоерку того, что запрашивавется фото
	if let Some(photo_name) = name	{
		let strok = format!("/home/roma/rust/juicy_site/avatars/{}", photo_name);
		return NamedFile::open(strok).await.ok()
	}
	None
}

#[derive(Debug, PartialEq, FromForm, Serialize)]
struct UserFromRequest<'a> {
    name: Option<&'a str>,
    nickname: Option<&'a str>,
    avatar: Option<&'a str>,
    role: Option<&'a str>,
    admin: Option<&'a str>,
    tg_id: Option<&'a str>,
    uuid: Option<&'a str>,
}

#[derive(Debug, PartialEq, FromForm)]
struct DocumentFromRequest<'a> {
    title: Option<&'a str>,
    path: Option<&'a str>,
    author: Option<UserFromRequest<'a>>,
    subject: Option<&'a str>,
    type_work: Option<&'a str>,
    number_work: Option<&'a str>,
}

//#[derive(Debug, Serialize, Clone)]
//enum Respone {
//    Users(Vec<user::User>),
//    Docs(Vec<Document>),
//    Nill
//}

#[derive(Debug, Serialize, Clone)]
struct Respone {
    Users: Vec<user::User>,
    Docs: Vec<Document>
}

#[get("/get?<user>&<doc>&<all_users>&<all_docs>")]
async fn get_val_new(
    user: Option<UserFromRequest<'_>>,
    doc: Option<DocumentFromRequest<'_>>,
    all_users: Option<bool>,
    all_docs: Option<bool>
) -> Json<Respone> {

    let mut respone = Respone{Users: Vec::with_capacity(4), Docs: Vec::with_capacity(4)};

    // Работа с поиском пользователей
    if let Some(true) = all_users {
        let db = DataBase::new(PATH_BD);

        if let Some(user_vec) = db.get_user(HashMap::new()) {
            respone.Users.extend_from_slice(&user_vec);
        }

    } else {
        let mut result_user_vec: Vec<user::User> = Vec::with_capacity(4);
        if let Some(user) = user {
            respone.Users.extend_from_slice(&check_user(user));
        }
    }

    // Работа с поиском документов
    if let Some(true) = all_docs {
        let db = DataBase::new(PATH_BD);

        if let Some(docs_vec) = db.get_doc(&HashMap::<&str, &str>::new()) {
            respone.Docs.extend_from_slice(&docs_vec);
        }

    } else {
        let mut result_doc_vec: Vec<Document> = Vec::with_capacity(4);
        if let Some(doc) = doc {
            respone.Docs.extend_from_slice(&check_doc(doc));
        }
    }

    return Json(respone);
}

/*
#[get("/get?<user>&<doc>&<all_users>&<all_docs>")]
async fn get_val(
    user: Option<UserFromRequest<'_>>,
    doc: Option<DocumentFromRequest<'_>>,
    all_users: Option<bool>,
    all_docs: Option<bool>
) -> Json<Vec<ResponeDocUser>> {

    let mut result_vec: Vec<ResponeDocUser> = Vec::with_capacity(4);

    if let Some(user) = user {
        result_vec.extend_from_slice(&check_user(user));
    }
    if let Some(doc) = doc {
        result_vec.extend_from_slice(&check_doc(doc));
    }

    if let Some(true) = all_users {
        let mut vec_result_user: Vec<ResponeDocUser> = Vec::new();
        let db = DataBase::new(PATH_BD);

        if let Some(user_vec) = db.get_user(HashMap::new()) {
            for user in user_vec {
                let tmp = user.clone();
                vec_result_user.push(ResponeDocUser::ResponeUser(tmp))
            }
            return Json(vec_result_user)
        }
    }

    if let Some(true) = all_docs {
        let mut vec_result_doc: Vec<ResponeDocUser> = Vec::new();
        let mut hm = HashMap::new();
        let db = DataBase::new(PATH_BD);

        if let Some(doc_vec) = db.get_doc(hm) {
            for doc in doc_vec {
                let tmp = doc.clone();
                vec_result_doc.push(ResponeDocUser::ResponeDoc(tmp))
            }
            return Json(vec_result_doc)
        }
    }

    Json(result_vec)
}

 */

fn check_user(user: UserFromRequest) -> Vec<user::User>{
    println!(" Пользователь {:?}", user);
    let mut hm = HashMap::new();

    if let Some(name) = user.name {
        hm.insert("name", name);
    }
    if let Some(nickname) = user.nickname {
        hm.insert("nickname", nickname);
    }
    if let Some(avatar) = user.avatar {
        hm.insert("avatar", avatar);
    }
    if let Some(role) = user.role {
        hm.insert("role", role);
    }
    if let Some(admin) = user.admin {
        hm.insert("admin", admin);
    }
    if let Some(tg_id) = user.tg_id {
        hm.insert("tg_id", tg_id);
    }
    if let Some(uuid) = user.uuid {
        hm.insert("uuid", uuid);
    }

    if hm.len() != 0 {  // Если никакие данные не были вставлены,
                        // значит пользователь не запрашивался
        let db = DataBase::new(PATH_BD);
        if let Some(user_vec) = db.get_user(hm) {
            return user_vec
        }
    }
    return Vec::new()
}


fn check_doc(doc: DocumentFromRequest) -> Vec<Document>{
    println!(" Документ {:?}", doc);

    let mut hm = HashMap::new();

    if let Some(title) = doc.title {
        hm.insert("title", title);
    }
    if let Some(path) = doc.path {
        hm.insert("path", path);
    }
    let mut vec_users_uuid = Vec::with_capacity(2);
    if let Some(author) = doc.author {
        /*
            Сначала получаем вектор из пользователей, документы который будем искать.
            Далее итерируя каждого пользователя ищем документы, которые имеют author_uuid равный user.uuid
            Найденные документы добавляем к релизному вектору
        */
        for user in check_user(author) {
            vec_users_uuid.push(user.uuid);
            //let db = DataBase::new(PATH_BD);

            //let tmp: HashMap<&str, &str> = HashMap::from([("author_uuid", user.uuid.as_str())]);
            //if let Some(doc_vec) = db.get_doc(tmp) {
            //    for doc in doc_vec {
            //        let tmp = doc.clone();
            //        vec_result_doc.push(ResponeDocUser::ResponeDoc(tmp));
            //    }
            //}

        }
    }
    if let Some(subject) = doc.subject {
        hm.insert("subject", subject);
           }
    if let Some(type_work) = doc.type_work {
        hm.insert("type_work", type_work);
    }
    if let Some(number_work) = doc.number_work {
        hm.insert("number_work", number_work);
    }

    let db = DataBase::new(PATH_BD);
    let mut res_doc_vec = Vec::new();

    if vec_users_uuid.len() != 0 {
        for user_uuid in &vec_users_uuid {
            hm.insert("author_uuid", user_uuid);
            if let Some(doc_vec) = db.get_doc(&hm) {
                res_doc_vec.extend_from_slice(&doc_vec)
            }
        }
        return res_doc_vec
    }

    if hm.len() != 0 {
        if let Some(doc_vec) = db.get_doc(&hm) {
            res_doc_vec.extend_from_slice(&doc_vec)
        }
        return res_doc_vec
    }
    return Vec::new();
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Task<'r> {
    description: &'r str,
    complete: bool
}


#[post("/todo", data = "<task>")]
fn new_doc(task: Json<Document>) -> Json<String>{
    println!("{:?}", task);
    //получить документ
    return Json(String::from("hello"));
}


#[macro_use] extern crate rocket;
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, icon])
        .mount("/api", routes![all_api, get_files])//, get_val, new_doc])
        .mount("/api_beta", routes![get_val_new])
}
