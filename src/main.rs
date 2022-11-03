mod db_conn;
use db_conn::{Document, User};

mod user_request;
use user_request::UserFromRequest;

mod document_request;
use document_request::DocumentFromRequest;

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf}
};

use rocket_sync_db_pools::{
    Connection,
    database,
    rusqlite::{
        self,
        params
    }
};

use rocket::{
    serde::{Serialize, Deserialize, json::Json},
    fs::{NamedFile, TempFile},
    Data,
    form::{Form, FromForm, FromFormField},
    data::{ToByteUnit, FromData, DataStream},
};
use rocket::http::tls::rustls::internal::msgs::enums::ContentType;

//Структура для возвращения пользователей и(или) документов
#[derive(Debug, Serialize, Clone)]
struct Response {
    users: Vec<db_conn::User>,
    docs: Vec<db_conn::Document>
}

/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<NamedFile> {
    NamedFile::open("icon_site.ico").await.ok()
}

/// Главная страница сайта
#[get("/")]
async fn index() -> Json<bool> {
    Json(true)
}

/// Хранит название запросов и их описание
#[derive(Serialize)]
struct AllApi<'a> {
    methods: Vec<&'a str>,
    about: Vec<&'a str>,
}

///Адрес для отображения списка api адресов
#[get("/")]
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
const DOCUMENTS_FORMAT: [&str; 3] = ["docx", "doc", "pdf"];
#[get("/get_file/<file_path..>")]
pub async fn get_files(file_path: PathBuf) -> Option<NamedFile> {

    println!("Запрошен файл по пути: {:?}", &file_path);

    let file_type =
        match file_path.extension() { //Если в пути есть формат файла
            Some(val) => val,
            None => return None
        };

    let mut path_dir = " ";

    // Соответсвует ли формат файла изображению
    for format in IMAGE_FORMAT {
        if *format == *file_type {
            path_dir = PATH_FOR_SAVE_AVATARS;
        }
    }

    // Соответсвует ли формат файла документу
    for format in DOCUMENTS_FORMAT {
        if *format == *file_type {
            path_dir = PATH_FOR_SAVE_DOCS;
        }
    }

    // Если формат файла не был опознан
    if path_dir == " " {
        return None;
    }

    return
        NamedFile::open(
            Path::new(path_dir)
                .join(file_path)
        ).await.ok()  //Возвращает файл или None
}

#[get("/get?<user>&<doc>&<all_users>&<all_docs>")]
async fn get_val_new<'a>(
    db: Db,
    user: Option<UserFromRequest<'a>>,
    doc: Option<DocumentFromRequest<'a>>,
    all_users: Option<bool>, // Если нужны все пользователи
    all_docs: Option<bool>  // Если нужны все документы
) -> Json<Response> {

    let mut response = Response {
        users: Vec::with_capacity(4),
        docs: Vec::with_capacity(4)
    };

    if let Some(true) = all_users { // Если потребовались все пользователи
        let res = db
            .run(
                move |conn| {
                    db_conn::get_all_user(conn)
                }
            ).await;
        response.users = res;
    } else { //Если необходимы пользователим по ключевым полям

        if let Some(user_v) = user {

            // Получаем HashMap типа <Данные_пользователя, запрашиваемое_значение>
            let hm = user_v.check_user();

            //Если запрос не с пустыми полями
            if hm.len() != 0 {
                if let Some(user_vec) = db.run(
                    |conn| db_conn::get_user(conn, hm)
                ).await
                {
                    response.users = user_vec
                }
            }
        }
    }

    // Работа с поиском документов
    if let Some(true) = all_docs { // Если нужны все документы
        response.docs = db.run(
            |conn| db_conn::get_doc(
                (HashMap::new(), None),
                conn
            )
        ).await;

    } else {
        // Если необходимы выбранные документы
        if let Some(doc) = doc {
            let tmp = doc.check_doc();
            // Если были введены поля для документа ИЛИ для автора документа
            if (tmp.0.len() != 0) || (tmp.1 != None) {
                response.docs = db.run(
                    move |conn| db_conn::get_doc(tmp, conn)
                ).await;
            }
        }
    }
    Json(response)
}


#[derive(Debug, FromForm)]
pub struct DocumentFile<'a> {
    pub title: String,
    pub file: TempFile<'a>,
    pub file_type: String,
    pub author_uuid: String,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
}

impl<'a> DocumentFile<'a> {
    async fn docfile_to_doc(&mut self) -> db_conn::Document {
        use uuid::Uuid;

        let doc_uuid = Uuid::new_v4().to_string();
        let file_name = format!("{}.{}", doc_uuid, self.file_type);
        let path = format!("{}{}", PATH_FOR_SAVE_DOCS, file_name);
        self.file.copy_to(path).await.unwrap();

        Document {
            title: self.title.clone(),
            path: file_name,
            author: User {
                name: "".to_string(),
                nickname: "".to_string(),
                avatar: "".to_string(),
                role: "".to_string(),
                admin: "".to_string(),
                tg_id: 0,
                uuid: self.author_uuid.clone()
            },
            subject: self.subject.clone(),
            type_work: self.type_work.clone(),
            number_work: self.number_work,
            note: self.note.clone(),
            doc_uuid: Some(doc_uuid)
        }
    }
}

#[post("/add_doc", data= "<file>")]
async fn new_doc(db: Db, mut file: Form<DocumentFile<'_>>) -> Json<String> {

    let mut filed = file.docfile_to_doc().await;
    let tmp = filed.path.clone();
    println!("{:?}", filed);

    db.run(|conn| {

        if db_conn::add_doc(conn, filed) {
            return Json(true);
        }
        else {
            return Json(false);
        }
    }).await;
    return Json(tmp);
}

#[delete("/del_doc?<doc_uuid>")]
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

#[put("/update_doc?<doc_uuid>", data="<new_doc>")]
async fn update_document(db: Db, doc_uuid: String, new_doc: Form<DocumentFromRequest<'_>>) -> Json<bool>{
    let hm_do = new_doc.into_inner().check_doc();
    println!("{:?}", &hm_do);

    Json(db.run(move |conn| {db_conn::update_doc(conn, hm_do.0, doc_uuid) }).await)
}

const PATH_FOR_SAVE_DOCS: &str = r"F:\";
const PATH_FOR_SAVE_AVATARS: &str = r"F:\Projects\Rust\juicy_site\avatars\";


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
                new_doc,
                update_document
            ]
        )
        .attach(
            Db::fairing()
        )
}
