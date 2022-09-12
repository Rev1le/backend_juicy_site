use rocket::http::ContentType;
use rocket::serde::{Serialize,json::{Json}};
use rocket::fs::{NamedFile};
use std::fs;
use std::collections::HashMap;

extern crate reqwest;

mod sqlite_conn;

use crate::sqlite_conn::user::{User};
use crate::sqlite_conn::document::Document;
use crate::sqlite_conn::DataBase;

const PATH_BD: &str = r"F:\Projects\Rust\juicy_site\test.db";

#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<NamedFile> {
    NamedFile::open("icon_site.ico").await.ok()
}

#[get("/")]
fn index() -> (ContentType, String) {
    let html = fs::read_to_string("/home/roma/PythonApps/dsBot/db_html.html").unwrap();
    (ContentType::HTML, html)
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

#[derive(Debug, PartialEq, FromForm)]
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

#[get("/")] // Для отображения списка api адресов
async fn all_api() -> Json<Vec<&'static str>> {
    Json(vec!["/get_photo", "/upload_files?<url>", "/get"])
}

#[get("/get_all_users")]
fn get_all_users() -> Json<Option<Vec<User>>> {
    let db = DataBase::new(PATH_BD);
    Json(Some(db.get_all_user()))
}

#[derive(Debug, Serialize, Clone)]
enum ResponeDocUser {
    ResponeUser(User),
    ResponeDoc(Document)
}


#[get("/get?<user>&<doc>")]
async fn get_val(
    user: Option<UserFromRequest<'_>>,
    doc: Option<DocumentFromRequest<'_>>)
    -> Json<Vec<ResponeDocUser>> {

    let mut result_vec: Vec<ResponeDocUser> = Vec::with_capacity(4);
    result_vec.extend_from_slice(&check_user(user));
    result_vec.extend_from_slice(&check_doc(doc));

    Json(result_vec)

}

fn check_user(user: Option<UserFromRequest>) -> Vec<ResponeDocUser>{
    let mut vec_result_user: Vec<ResponeDocUser> = Vec::new();
    let mut clear_user = true;
    match user {
        None => {},
        Some(user) => {
            println!(" Пользователь {:?}", user);
            let mut hm = HashMap::new();
            if let Some(name) = user.name {
                hm.insert("name", name);
                clear_user = false;
            }
            if let Some(nickname) = user.nickname {
                hm.insert("nickname", nickname);
                clear_user = false;
            }
            if let Some(avatar) = user.avatar {
                hm.insert("avatar", avatar);
                clear_user = false;
            }
            if let Some(role) = user.role {
                hm.insert("role", role);
                clear_user = false;
            }
            if let Some(admin) = user.admin {
                hm.insert("admin", admin);
                clear_user = false;
            }
            if let Some(tg_id) = user.tg_id {
                hm.insert("tg_id", tg_id);
                clear_user = false;
            }
            if let Some(uuid) = user.uuid {
                hm.insert("uuid", uuid);
                clear_user = false;
            }
            if !clear_user {
                let db = DataBase::new(PATH_BD);
                if let Some(user_vec) = db.get_user(hm) {
                    for user in user_vec {
                        let tmp = user.clone();
                        vec_result_user.push(ResponeDocUser::ResponeUser(tmp))
                    }
                }
            }

        }
    }
    vec_result_user
}

fn check_doc(doc: Option<DocumentFromRequest>) -> Vec<ResponeDocUser>{
    let mut vec_result_doc: Vec<ResponeDocUser> = Vec::new();
    let mut clear_doc = true;
    match doc {
        None => {},
        Some(doc) => {
            println!(" Документ {:?}", doc);
            let mut hm = HashMap::new();
            if let Some(title) = doc.title {
                hm.insert("title", title);
                clear_doc = false;
            }
            if let Some(path) = doc.path {
                hm.insert("path", path);
                clear_doc = false;
            }
             if let Some(author) = doc.author {
                 let respone_users_vec = check_user(Some(author));
                 /*
                 Сначала получаем вектор из пользователей, документы который будем искать.
                 Далее итерируя каждого пользователя ищем документы, которые имеют author_uuid равный user.uuid
                 Найденные документы добавляем к релизному вектору
                  */
                 for respone_user in respone_users_vec {
                     let db = DataBase::new(PATH_BD);
                     if let ResponeDocUser::ResponeUser(user) = respone_user {
                         let tmp: HashMap<&str, &str> = HashMap::from([("author_uuid", user.uuid.as_str())]);
                         if let Some(doc_vec) = db.get_doc(tmp) {
                             for doc in doc_vec {
                                 let tmp = doc.clone();
                                 vec_result_doc.push(ResponeDocUser::ResponeDoc(tmp));
                             }
                         }
                     }
                 }
             }
            if let Some(subject) = doc.subject {
                hm.insert("subject", subject);
                clear_doc = false;
            }
            if let Some(type_work) = doc.type_work {
                hm.insert("type_work", type_work);
                clear_doc = false;
            }
            if let Some(number_work) = doc.number_work {
                hm.insert("number_work", number_work);
                clear_doc = false;
            }

            if !clear_doc {
                let db = DataBase::new(PATH_BD);
                if let Some(doc_vec) = db.get_doc(hm) {
                    for doc in doc_vec {
                        let tmp = doc.clone();
                        vec_result_doc.push(ResponeDocUser::ResponeDoc(tmp));
                    }
                }
            }
        }
    }
    vec_result_doc

}

#[macro_use] extern crate rocket;
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, icon])
    .mount("/api", routes![all_api, get_files, get_val, get_all_users])
}
