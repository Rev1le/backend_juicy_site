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

use crate::{api, CONFIG, Config, Db};

struct CacheDocuments(Mutex<Vec<Document>>);
struct CacheUsers(Mutex<Vec<Document>>);

struct ApiCache {
    documents: Mutex<HashMap<String, Document>>,
    users: Mutex<HashMap<String, User>>,
}

impl ApiCache {
    // Добавить бинарный поиск при поомщи реализайии трейта Ord
    // и сравнения уникльного айди документа
    async fn get_docs(&self) -> Vec<Document> {
        self.documents.lock().await.values().map(|doc| doc.clone()).collect()
    }

    async fn get_users(&self) -> Vec<User> {
        self.users.lock().await.values().map(|user| user.clone()).collect()
    }

    async fn get_doc_by_uuid(&self, doc_uuid: &str) -> Option<Document> {
        self.documents.lock().await.get(doc_uuid).cloned()
    }

    async fn append_new_doc(&self, doc: Document) {
        self.documents.lock().await.insert(doc.doc_uuid.clone(), doc);
    }

    async fn append_new_user(&self, user: User) {
        self.users.lock().await.insert(user.uuid.clone(), user);
    }

    async fn remove_doc(&self, doc_uuid: &str) {
        self.documents.lock().await.remove(doc_uuid);
    }

    async fn remove_user(&self, user_uuid: &str) {
        self.users.lock().await.remove(user_uuid);
    }
}

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
    state: &State<CacheDocuments>,
    cache: &State<ApiCache>,
    db: Db,
    user: UserFromRequest<'a>,
    doc: DocumentFromRequest<'a>,
    all_users: Option<bool>, // Если нужны все пользователи
    all_docs: Option<bool>,  // Если нужны все документы
    no_cache: bool,
) -> Json<Response> {

    println!("{:?}", cache.documents);

    let mut response = Response {
        users: Vec::with_capacity(10),
        docs: Vec::with_capacity(10)
    };

    if let Some(true) = all_users { // Если потребовались все пользователи
        let res = db
            .run(
                move |conn| {
                    db_conn::get_user(conn, HashMap::new())
                        .expect("Ошибка при получении всех пользователей с ошибкой")
                }
            ).await;
        if res.len() > 0 {
            response.users = res;
        }
    } else
    //Если необходимы пользователим по ключевым полям
    {

        // Получаем HashMap типа <Данные_пользователя, запрашиваемое_значение>
        let hm = user.to_hashmap();

        //Если запрос не с пустыми полями
        if hm.len() != 0 {
            let user_vec = db.run(
                |conn| db_conn::get_user(conn, hm)
            ).await.expect("Ошибка при получении пользователей по пармаетрам");

            if user_vec.len() > 0 {
                response.users = user_vec
            }
        }
    }

    // Работа с поиском документов
    if let Some(true) = all_docs { // Если нужны все документы
        let mut mutex = state.inner().0.lock().await;

        if no_cache {
            println!("Документы были запрошены с БД");

            response.docs = db.run(
                |conn| db_conn::get_doc(
                    HashMap::new(),
                    None,
                    conn
                )
            ).await.expect("Ошибка при выводе всех документов");
            //*mutex = response.docs.clone();

            for doc in response.docs.iter() {
                cache.append_new_doc(doc.clone()).await;
            }

        } else {
            println!("Документы были получены из кеша");
            println!("{:?}", cache.get_docs().await);
            //response.docs = mutex.clone();
            response.docs = cache.get_docs().await.clone();
        }

    } else {
        // Если необходимы выбранные документы
        let hashmap_doc = doc.to_hashmap();
        let hashmap_author = doc.author_to_hashmap();

        // Если были введены поля для документа ИЛИ для автора документа
        if (hashmap_doc.len() != 0) || (hashmap_author.len() != 0) {
            response.docs = db.run(
                move |conn| db_conn::get_doc(hashmap_doc, Some(hashmap_author), conn)
            ).await.expect("Ошибка при выводе документов по параметрам");
        }
    }
    Json(response)
}

//При добавлени пользователя и взятии из кеша поля юзера пустые
#[post("/add_doc", data= "<file>")]
async fn new_doc(
    state: &State<CacheDocuments>,
    cache: &State<ApiCache>,
    db: Db,
    mut file: Form<DocumentFile<'_>>
) -> Json<String> {

    let filed = file.docfile_to_doc(&CONFIG.path_to_save_docs).await;
    let filed_cl = filed.clone();
    let doc_path = filed.path.clone();
    println!("{:?}", &filed);

    let added_doc: bool = db.run(|conn| {

        match db_conn::add_doc(conn, filed) {
            Ok(_) => true,
            Err(_) => false,
        }
    }).await;

    if !added_doc {
        return Json(String::from("Ошибка добавления документа"));
    }

    cache.append_new_doc(filed_cl).await;
    //state.inner().0.lock().await.push(filed_cl);
    return Json(doc_path);
}

#[delete("/del_doc?<doc_uuid>")]
async fn delete_document(state: &State<CacheDocuments>, cache: &State<ApiCache>, db: Db, doc_uuid: String) -> Json<bool> {

    let path = CONFIG.path_to_save_docs.clone();
    let doc_uuid_tmp = doc_uuid.clone();

    let res_deleted: bool =  db.run(move |conn| {
        db_conn::del_doc(
            conn,
            &path,
            &doc_uuid
        )
    }).await;

    if !res_deleted {
        println!("Удаение не было произвдеено.");
        return Json(false)
    }

    if cache.documents.lock().await.remove(&doc_uuid_tmp) != None {
        println!("Документ был удален из кеша");
        return Json(true)
    }
    println!("Файла не оказолось в кеше");
    return Json(false);
}

// Кеш не ипортирован
#[put("/update_doc?<doc_uuid>", data="<new_doc>")]
async fn update_document(
    cache: &State<ApiCache>,
    db: Db,
    doc_uuid: String,
    new_doc: Form<DocumentFromRequest<'_>>
) -> Json<bool>{

    let hm_do = new_doc.into_inner().to_hashmap();
    let clon_doc_uuid = doc_uuid.clone();

    let updated_doc = db.run(
        move |conn| db_conn::update_doc(conn, hm_do, doc_uuid)
    ).await;

    if updated_doc {
        cache.remove_doc(&clon_doc_uuid);
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
            .manage(CacheDocuments(Mutex::new(Vec::default())))
            .manage(
                ApiCache {
                    documents: Mutex::new(HashMap::default()),
                    users: Mutex::new(HashMap::default()),
                }
            )
    })
}