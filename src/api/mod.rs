use rocket::{
    State,
    fs::NamedFile,
    form::Form,
    fairing::AdHoc,
    serde::{
        json::Json,
        Serialize,
    },
    tokio::sync::Mutex};

use std::{
    collections::HashMap,
    path::{Path, PathBuf}
};
use once_cell::sync::Lazy;
use rusqlite::Error;

use user::{
    User,
    UserFromRequest
};

use document::{
    Document,
    DocumentFile,
    DocumentFromRequest,
};

pub mod user;
pub mod document;
pub mod db_conn;
mod api_cache;

use crate::{api, CONFIG, Config, Db};
use crate::api::api_cache::ApiCache;
use crate::api::db_conn::get_all_users_uuid;

struct CacheDocuments(Mutex<Vec<Document>>);
struct CacheUsers(Mutex<Vec<Document>>);

//Структура для возвращения пользователей и(или) документов
#[derive(Debug, Serialize, Clone)]
struct Response {
    users: Vec<User>,
    docs: Vec<Document>
}

/// Хранит название запросов и их описание
#[derive(Serialize)]
struct AllApi<'a> {
    methods: Vec<&'a str>,
    about: Vec<&'a str>,
}

///Адрес для отображения списка api адресов
#[get("/")]
async fn all_api<'a>(cache: &State<ApiCache>) -> Json<AllApi<'a>> {
    cache.inner().write_cache_to_json().await;
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

#[get("/get_file/<file_name..>")]
pub async fn get_files(file_name: PathBuf) -> Option<NamedFile> {

    println!("Запрошен файл по пути: {:?}", &file_name);

    let type_file =
        match file_name.extension() { //Если в пути есть формат файла
            Some(val) => val,
            None => return None
        };

    let mut path_dir = "".to_string();

    // Соответсвует ли формат файла изображению
    for format in IMAGE_FORMAT {
        if *format == *type_file {
            path_dir.push_str(&CONFIG.path_to_save_img);
        }
    }

    // Соответсвует ли формат файла документу
    for format in DOCUMENTS_FORMAT {
        if *format == *type_file {
            path_dir.push_str(&CONFIG.path_to_save_docs);
        }
    }

    // Если формат файла не был опознан
    if path_dir == "" {
        return None;
    }

    return
        NamedFile::open(
            Path::new(&path_dir)
                .join(file_name)
        ).await.ok()  //Возвращает файл или None
}

#[get("/get?<user>&<doc>&<all_users>&<all_docs>&<no_cache>")]
async fn get_json_user_doc<'a>(
    cache: &State<api_cache::ApiCache>,
    db: Db,
    user: UserFromRequest<'a>,
    doc: DocumentFromRequest<'a>,
    all_users: bool, // Если нужны все пользователи
    all_docs: bool,  // Если нужны все документы
    no_cache: bool,
) -> Json<Response> {

    let mut response = Response {
        users: Vec::with_capacity(10),
        docs: Vec::with_capacity(10)
    };

    if no_cache {

        match (all_users, all_docs) {

            (true, true) => {
                response.users = user::get_all_users(&db).await;
                response.docs = document::get_all_docs(&db).await;
            },

            (true, _) => response.users = user::get_all_users(&db).await,
            (_, true) => response.docs = document::get_all_docs(&db).await,

            (false, false) => {
                response.users = user.get_users_db(&db).await;
                response.docs = doc.get_docs_db(&db).await;
            },
        }

        cache.set_users(&response.users).await;
        cache.set_docs(&response.docs).await;

    } else {

        match (all_users, all_docs) {

            (true, true) => {
                response.users = cache.get_users().await;
                response.docs = cache.get_docs().await;
            },

            (true, _) => response.users = cache.get_users().await,
            (_, true) => response.docs = cache.get_docs().await,

            (false, false) => {
                response.users = user.get_users_db(&db).await;
                response.docs = doc.get_docs_db(&db).await;
            },
        }
    }

    Json(response)
}

//При добавлени пользователя и взятии из кеша поля юзера пустые
#[post("/add_doc", data= "<file>")]
async fn new_doc(
    cache: &State<api_cache::ApiCache>,
    db: Db,
    mut file: Form<DocumentFile<'_>>
) -> Json<String> {

    let filed = file.docfile_to_doc(&CONFIG.path_to_save_docs).await;
    let filed_cl = filed.clone();
    let doc_path = filed.path.clone();
    println!("{:?}", &filed);

    let added_doc: bool = db.run(|conn| {
        db_conn::add_doc(conn, filed).is_ok()
    }).await;

    if !added_doc {
        return Json(String::from("Ошибка добавления документа"));
    }

    cache.append_doc(Document {
        author: cache.get_user_by_uuid(&filed_cl.author.uuid).await.unwrap(),
        ..filed_cl
    }).await;

    return Json(doc_path);
}

#[delete("/del_doc?<doc_uuid>")]
async fn delete_document(cache: &State<api_cache::ApiCache>, db: Db, doc_uuid: String) -> Json<bool> {

    let path = CONFIG.path_to_save_docs.clone();
    let doc_uuid_tmp = doc_uuid.clone();

    let res_deleted: bool =  db.run(move |conn| {
        db_conn::del_doc(
            conn,
            &path,
            &doc_uuid
        )
    }).await;

    let cache_deleted = cache.remove_doc(&doc_uuid_tmp).await;

    if res_deleted && cache_deleted.is_some() {
        println!("Документ был удален из кеша");
        return Json(true)
    }

    println!("Файла не был удален удаление в бд: {} \nудаление в кеше: {}", res_deleted, cache_deleted.is_some());
    return Json(false);
}

// Кеш не ипортирован
#[put("/update_doc?<doc_uuid>", data="<new_doc>")]
async fn update_document(
    cache: &State<api_cache::ApiCache>,
    db: Db,
    doc_uuid: String,
    new_doc: Form<DocumentFromRequest<'_>>
) -> Json<bool>{

    let hm_do = new_doc.into_inner().to_hashmap();
    let clon_doc_uuid = doc_uuid.clone();

    let cl_gm_do = hm_do.clone();

    let updated_doc = db.run(
        move |conn| db_conn::update_doc(conn, cl_gm_do, doc_uuid)
    ).await;

    if updated_doc {
        let mut tmp_doc = cache.remove_doc(&clon_doc_uuid).await.unwrap();

        for param in hm_do {
            match param.0.as_str(){
                "title" => {
                    tmp_doc.title = param.0;
                    continue;
                },
                "path" => {
                    tmp_doc.path = param.0;
                    continue;
                },
                "subject" => {
                    tmp_doc.subject = param.0;
                    continue;
                },
                "type_work" => {
                    tmp_doc.type_work = param.0;
                    continue;
                },
                "number_work" => {
                    tmp_doc.number_work = param.0.parse::<i64>().unwrap();
                    continue;
                },
                _ => {
                    continue;
                }
            }
        }

        cache.append_doc(tmp_doc);
    }

    Json(false)
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("API stage", |rocket| async {
        rocket
            .mount(
                "/api",
                routes![
                    all_api,            // Спиок всех API url путей
                    get_files,          // Получение файла
                    delete_document,    // Удаление документа
                    get_json_user_doc,  // Получение данных из БД
                    new_doc,            // Создание документа
                    update_document     // Обновление сведений об документе
                ]
            )
            //.manage(CacheDocuments(Mutex::new(Vec::default())))
            .manage(api_cache::ApiCache::new())
            .attach(api_cache::state())
    })
}