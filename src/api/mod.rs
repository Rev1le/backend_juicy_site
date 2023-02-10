pub mod user;
pub mod document;
mod api_cache;

use std::{
    path::{Path, PathBuf}
};
use rocket::{
    State,
    fs::NamedFile,
    form::Form,
    fairing::AdHoc,
    http::{Cookie, CookieJar, Status},
    serde::{
        json::Json,
        Serialize,
    }
};

use user::{
    User,
    UserFromRequest
};
use document::{
    Document,
    DocumentFile,
    DocumentFromRequest,
};

use crate::{db_conn, CONFIG, Db,
            user_account::{CacheSessions, StateAuthUser, DataAccess}
};
use api_cache::ApiCache;

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
const DOCUMENTS_FORMAT: [&str; 6] = ["docx", "doc", "pdf", "txt", "pptx", "exel"];

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
    cache: &State<ApiCache>,
    db: Db,
    user: UserFromRequest<'a>,
    doc: DocumentFromRequest<'a>,
    all_users: bool, // Если нужны все пользователи
    all_docs: bool,  // Если нужны все документы
    no_cache: bool,
) -> Json<Response> {

    let mut response = Response {
        users: Vec::default(),
        docs: Vec::default(),
    };

    // Если требуются данные из БД
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

        // Обновляем Хеш-таблицу
        cache.set_users(&response.users).await;
        cache.set_docs(&response.docs).await;

        return Json(response)
    }

    match (all_users, all_docs) {
        (true, true) => {
            response.users = cache.get_users().await;
            response.docs = cache.get_docs().await;
        },

        (true, _) => response.users = cache.get_users().await,
        (_, true) => response.docs = cache.get_docs().await,

        // Если нужны не все пользователи/документы то запрашиваем из БД по нужным полям
        (false, false) => {
            response.users = user.get_users_db(&db).await;
            response.docs = doc.get_docs_db(&db).await;
        },
    }

    Json(response)
}

//При добавлени пользователя и взятии из кеша поля юзера пустые
#[post("/add_doc", data= "<file>")]
async fn new_doc(
    cache: &State<api_cache::ApiCache>,
    sessions: &State<CacheSessions>,
    cookies: &CookieJar<'_>,
    db: Db,
    mut file: Form<DocumentFile<'_>>
) -> Json<DataAccess<String, ()>> {

    if !user_access(cookies, sessions).await {
        return Json(DataAccess::Denied(()))
    }

    let filed = file.docfile_to_doc(&CONFIG.path_to_save_docs).await;
    let doc_path = filed.path.clone();
    println!("{:?}", &filed);

    let filed_cl = filed.clone();
    let added_doc: bool = db.run(
        |conn| db_conn::add_doc(conn, filed_cl).is_ok()
    ).await;

    if !added_doc {
        return Json(DataAccess::Allowed(String::from("Ошибка добавления документа")));
    }

    // Елси пользователя нет в кеше - паника
    cache.append_doc(Document {
        author: cache.get_user_by_uuid(&filed.author.uuid).await.unwrap(),
        ..filed
    }).await;

    return Json(DataAccess::Allowed(doc_path));
}

#[delete("/del_doc?<doc_uuid>")]
async fn delete_document(
    cache: &State<ApiCache>,
    sessions: &State<CacheSessions>,
    cookies: &CookieJar<'_>,
    db: Db,
    doc_uuid: String
) -> Json<DataAccess<bool, ()>> {

    if !user_access(cookies, sessions).await {
        return Json(DataAccess::Denied(()))
    }

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
        return Json(DataAccess::Allowed(true))
    }

    println!("Файла не был удален в: удаление в бд: {} \nудаление в кеше: {}", res_deleted, cache_deleted.is_some());
    return Json(DataAccess::Allowed(false));
}

#[put("/update_doc?<doc_uuid>", data="<new_doc>")]
async fn update_document(
    cache: &State<api_cache::ApiCache>,
    sessions: &State<CacheSessions>,
    cookies: &CookieJar<'_>,
    db: Db,
    doc_uuid: String,
    new_doc: Form<DocumentFromRequest<'_>>
) -> Json<DataAccess<bool, ()>> {

    if !user_access(cookies, sessions).await {
        println!("Доступ был запрещен");
        return Json(DataAccess::Denied(()))
    }

    let hm_do = new_doc.into_inner().to_hashmap();
    let clon_doc_uuid = doc_uuid.clone();

    let cl_gm_do = hm_do.clone();

    match cache.remove_doc(&clon_doc_uuid).await {
        None => return Json(DataAccess::Allowed(false)),
        Some(mut tmp_doc) => {
            let updated_doc = db.run(
                move |conn| db_conn::update_doc(conn, cl_gm_do, doc_uuid)
            ).await;

            if updated_doc {

                for param in hm_do {
                    match param.0.as_str() {
                        "title" => {
                            tmp_doc.title = param.1;
                        },
                        "path" => {
                            tmp_doc.path = param.1;
                        },
                        "subject" => {
                            tmp_doc.subject = param.1;
                        },
                        "type_work" => {
                            tmp_doc.type_work = param.1;
                        },
                        "number_work" => {
                            println!("Число для парсинга: {}", &param.1);
                            tmp_doc.number_work = param.1.parse::<i64>().unwrap();
                        },
                        _ => {}
                    }
                }
                cache.append_doc(tmp_doc).await;

                return Json(DataAccess::Allowed(true))
            }
            return Json(DataAccess::Allowed(false))
        }
    }
}

async fn user_access(cookies: &CookieJar<'_>, sessions: &CacheSessions) -> bool {

    if let Some(session) = cookies.get("session_token") {

        match sessions.get_user_authconfirm(session.value()).await {
            Some(user) => {
                println!("Документ удаляет {}", user.nickname);
                return true
            },
            None => return false
        }

    } else {
        return false
    }
}

pub fn state() -> AdHoc {
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
            .manage(ApiCache::new())
            .attach(api_cache::state())
    })
}