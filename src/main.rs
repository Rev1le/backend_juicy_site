mod db_conn;

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
};

use rocket_sync_db_pools::{
    database,
    rusqlite::{
        self,
        // Connection,
        params
    }
};
use rocket::{
    serde::{
        Serialize,
        Deserialize,
        json::Json
    },
    fs::{
        NamedFile
    },
    Data,
    data::ToByteUnit,
};
use crate::db_conn::User;


#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<NamedFile> {
    NamedFile::open("icon_site.ico").await.ok()
}

#[get("/")]
async fn index() -> Json<bool>{//(ContentType, String) {
    Json(true)
}

#[derive(Serialize)]
struct AllApi<'a> {
    methods: Vec<&'a str>,
    about: Vec<&'a str>,
}

#[get("/")] // Для отображения списка api адресов
async fn all_api<'a>() -> Json<AllApi<'a>> {
    Json(
        AllApi {
            methods: vec![
                "/get_photo",
                "/upload_files?<url>",
                "/get",
                "/add_doc",
                "/del_doc"
            ],
            about: vec![
                "Получение фото",
                "Загрузка файлов",
                "Получение Пользователей или Документов",
                "Добавление документа в БД",
                "Удаление документа из БД"
            ]
        }
    )
}

const IMAGE_FORMAT: [&str; 3] = ["ico", "png", "jpg"];
#[get("/get_file?<name>")]
pub async fn get_files(name: Option<&str>) -> Option<NamedFile> {
	if let Some(file_name) = name	{

         //file_name.split(".").last() {

		let strok = format!(
            "{}{}",
            PATH_FOR_SAVE_AVATARS,
            file_name.to_string()
        );

        return NamedFile::open(strok)
            .await
            .ok()
	}
	None
}

#[derive(Debug, FromForm, Copy, Clone)]
struct UserFromRequest<'a> {
    name: Option<&'a str>,
    nickname: Option<&'a str>,
    avatar: Option<&'a str>,
    role: Option<&'a str>,
    admin: Option<&'a str>,
    tg_id: Option<&'a str>,
    uuid: Option<&'a str>,
}

#[derive(Debug,  FromForm)]
struct DocumentFromRequest<'a> {
    title: Option<&'a str>,
    path: Option<&'a str>,
    author: Option<UserFromRequest<'a>>,
    subject: Option<&'a str>,
    type_work: Option<&'a str>,
    number_work: Option<&'a str>,
}

#[derive(Debug, Serialize, Clone)]
struct Respone {
    users: Vec<db_conn::User>,
    docs: Vec<db_conn::Document>
}

#[get("/get?<user>&<doc>&<all_users>&<all_docs>")]
async fn get_val_new<'a>(
    db: Db,
    user: Option<UserFromRequest<'a>>,
    doc: Option<DocumentFromRequest<'a>>,
    all_users: Option<bool>,
    all_docs: Option<bool>
) -> Json<Respone> {
    let mut respone = Respone {
        users: Vec::with_capacity(4),
        docs: Vec::with_capacity(4)
    };

    // Работа с поиском пользователей
    if let Some(true) = all_users {
        let res = db
            .run(move |conn| {
                db_conn::get_all_user(conn)
            });
        respone.users = res.await;
    } else {

        if let Some(user_v) = user {

            let hm = check_user(&user_v);
            let res = db
                .run(move |conn| {
                    if hm.len() != 0 {
                        if let Some(user_vec) = db_conn::get_user(conn,  hm) {
                            return user_vec
                        }
                    }
                    return Vec::new()
                });

            //let tmp = check_user(&user, &conn);
            respone.users = res.await;
        }
    }

    // Работа с поиском документов
    if let Some(true) = all_docs {
        let res = db
            .run(move |conn| {
                db_conn::get_doc((HashMap::new(), None), conn)
            });
        respone.docs = res.await;

    } else {
        if let Some(doc) = doc {
            let tmp = check_doc(doc);
            if tmp.0.len() != 0 || tmp.1 != None {
                let res = db.run(move |conn| { db_conn::get_doc(tmp, conn)});
                respone.docs = res.await;
            }
        }
    }


    return Json(respone);
}

fn check_user(user: &UserFromRequest) -> HashMap<String, String> {
    //println!(" Пользователь {:?}", user);
    let mut hm = HashMap::new();

    if let Some(name) = user.name {
        hm.insert(
            "name".to_string(),
            name.to_string()
        );
    }
    if let Some(nickname) = user.nickname {
        hm.insert(
            "nickname".to_string(),
            nickname.to_string()
        );
    }
    if let Some(avatar) = user.avatar {
        hm.insert(
            "avatar".to_string(),
            avatar.to_string()
        );
    }
    if let Some(role) = user.role {
        hm.insert(
            "role".to_string(),
            role.to_string()
        );
    }
    if let Some(admin) = user.admin {
        hm.insert(
            "admin".to_string(),
            admin.to_string()
        );
    }
    if let Some(tg_id) = user.tg_id {
        hm.insert(
            "tg_id".to_string(),
            tg_id.to_string()
        );
    }
    if let Some(uuid) = user.uuid {
        hm.insert(
            "uuid".to_string(),
            uuid.to_string()
        );
    }
    hm
}

fn check_doc(doc: DocumentFromRequest)
    -> (
        HashMap<String, String>,
        Option<HashMap<String, String>>
    ) {
    //println!(" Документ {:?}", doc);
    let mut res: (HashMap<String, String>, Option<HashMap<String, String>>) = (HashMap::new(), None);

    if let Some(title) = doc.title {
        res.0.insert(
            "title".to_string(),
            title.to_string()
        );
    }
    if let Some(path) = doc.path {
        res.0.insert(
            "path".to_string(),
            path.to_string()
        );
    }
    if let Some(author) = doc.author {
        let tmp = check_user(&author);
        if tmp.len() != 0 {
            res.1 = Some(tmp);
        }
    }
    if let Some(subject) = doc.subject {
        res.0.insert(
            "subject".to_string(),
            subject.to_string()
        );
    }
    if let Some(type_work) = doc.type_work {
        res.0.insert(
            "type_work".to_string(),
            type_work.to_string()
        );
    }
    if let Some(number_work) = doc.number_work {
        res.0.insert(
            "number_work".to_string(),
            number_work.to_string()
        );
    }

    res
}

#[derive(Debug, Serialize, Deserialize, Clone, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct DocumentFile {
    pub title: String,
    pub file: Vec<u8>,
    pub file_type: String,
    pub author_uuid: String,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
}

#[post("/add_doc", data= "<file>")]
async fn new_doc(db: Db, file: Json<DocumentFile>) -> Json<bool>{

    db.run(move |conn| {
        let tmp = db_conn::get_all_users_uuid(conn)
            .iter()
            .position(|each|
                *each == file.author_uuid
            );

        return if let Some(_) = tmp {
            if db_conn::add_doc(conn, file) {
                Json(true)
            } else {
                Json(false)
            }
        } else {
            Json(false)
        }

    }).await
}

#[get("/del_doc?<doc_uuid>")]
async fn delete_document(db: Db, doc_uuid: String) -> Json<bool> {
    Json(
        db.run(move |conn| {
            db_conn::del_doc(
                conn,
                &doc_uuid
            )
        }).await
    )
}


const PATH_FOR_SAVE_DOCS: &str = r"C:\Users\nikiy\Documents\";
const PATH_FOR_SAVE_AVATARS: &str = r"C:\Users\nikiy\Desktop\backend_juicy_site\avatars\";


#[database("rusqlite")]
pub struct Db(rusqlite::Connection);

#[macro_use] extern crate rocket;
extern crate alloc;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                icon
            ]
        )
        .mount(
            "/api",
            routes![
                all_api,
                get_files,
                delete_document,
                get_val_new,
                new_doc
            ]
        )
        .mount(
            "/api_beta",
            routes![
                get_val_new,
                new_doc
            ]
        )
        .attach(
            Db::fairing()
        )
}
