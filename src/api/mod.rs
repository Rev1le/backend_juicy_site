use rocket::{fs::NamedFile, form::Form, fairing::AdHoc, serde::{
    json::Json,
    Serialize,
}, State};

use std::{
    sync::Mutex,
    collections::HashMap,
    path::{Path, PathBuf}
};
use std::string::ToString;

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

use crate::{api, Config, Db};
//use crate::Config;
struct CacheDocuments(rocket::tokio::sync::Mutex<Vec<Document>>);

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
pub async fn get_files(state: &State<Config>, file_name: PathBuf) -> Option<NamedFile> {

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
            path_dir.push_str(&state.path_to_save_img);
        }
    }

    // Соответсвует ли формат файла документу
    for format in DOCUMENTS_FORMAT {
        if *format == *type_file {
            path_dir.push_str(&state.path_to_save_docs);
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
    db: Db,
    user: UserFromRequest<'a>,
    doc: DocumentFromRequest<'a>,
    all_users: Option<bool>, // Если нужны все пользователи
    all_docs: Option<bool>,  // Если нужны все документы
    no_cache: bool,
) -> Json<Response> {

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
        if let Some(val) = res {
            response.users = val;
        } else {
            println!("Ошибка при получении всех документов");
        }
    } else
    //Если необходимы пользователим по ключевым полям
    {

        // Получаем HashMap типа <Данные_пользователя, запрашиваемое_значение>
        let hm = user.to_hashmap();

        //Если запрос не с пустыми полями
        if hm.len() != 0 {
            let opt_user_vec = db.run(
                |conn| db_conn::get_user(conn, hm)
            ).await.expect("Ошибка при получении пользователей по пармаетрам");

            if let Some(user_vec) = opt_user_vec {
                response.users = user_vec
            }
        }
    }

    // Работа с поиском документов
    if let Some(true) = all_docs { // Если нужны все документы
        let mut mutex = state.inner().0.lock().await;

        if !no_cache && mutex.len() != 0 {
            println!("Документы были получены из кеша");
            response.docs = mutex.clone();
        } else {
            println!("Документы были запрошены с БД");
            response.docs = db.run(
                |conn| db_conn::get_doc(
                    HashMap::new(),
                    None,
                    conn
                )
            ).await.expect("Ошибка при выводе всех документов");
            *mutex = response.docs.clone();
        }

    } else {
        // Если необходимы выбранные документы
        let hashmap_doc = doc.to_hashmap();
        let hashmap_author = doc.author_to_hashmap();

        println!("{:?}{:?}", &hashmap_doc, &hashmap_author);

        // Если были введены поля для документа ИЛИ для автора документа
        if (hashmap_doc.len() != 0) || (hashmap_author != None) {
            response.docs = db.run(
                move |conn| db_conn::get_doc(hashmap_doc, hashmap_author, conn)
            ).await.expect("Ошибка при выводе документов по параметрам");
        }
    }
    Json(response)
}




#[post("/add_doc", data= "<file>")]
async fn new_doc(state: &State<Config>, db: Db, mut file: Form<DocumentFile<'_>>) -> Json<String> {

    let filed = file.docfile_to_doc(&state.path_to_save_docs).await;
    let tmp = filed.path.clone();
    println!("{:?}", &filed);

    db.run(|conn| {
        return if let Ok(_) = db_conn::add_doc(conn, filed) {
            Json(true)
        } else {
            Json(false)
        }
    }).await;
    return Json(tmp);
}

#[delete("/del_doc?<doc_uuid>")]
async fn delete_document(state: &State<Config>, db: Db, doc_uuid: String) -> Json<bool> {

    let path = state.path_to_save_docs.clone();

    Json(
        db.run(move |conn| {
            db_conn::del_doc(
                &path,
                conn,
                &doc_uuid
            )
        }).await
    )
}

#[put("/update_doc?<doc_uuid>", data="<new_doc>")]
async fn update_document(
    db: Db,
    doc_uuid: String,
    new_doc: Form<DocumentFromRequest<'_>>
) -> Json<bool>{

    let hm_do = new_doc.into_inner().to_hashmap();
    println!("{:?}", &hm_do);

    Json(
        db.run(
            move |conn| db_conn::update_doc(conn, hm_do, doc_uuid)
        ).await
    )
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
            .manage(
                CacheDocuments(
                    rocket::tokio::sync::Mutex::new(Vec::with_capacity(15))
                )
            )
    })
}