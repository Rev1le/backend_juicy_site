pub mod user;
pub mod document;
pub mod db_conn;

use rocket::{
    fs::NamedFile,
    form::Form,
    fairing::AdHoc,
    serde::{
        json::Json,
        Serialize,
    }
};

use std::{
    collections::HashMap,
    path::{Path, PathBuf}
};

use user::{
    User,
    UserFromRequest
};

use document::{
    Document,
    DocumentFile,
    DocumentFromRequest
};

use crate::Db;
use crate::CONFIG;

// Пути для сохранения изображений и дркументов
const PATH_FOR_SAVE_DOCS: &str = CONFIG.path_to_save_docs;
const PATH_FOR_SAVE_AVATARS: &str = CONFIG.path_to_save_img;

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

    let mut path_dir = " ";

    // Соответсвует ли формат файла изображению
    for format in IMAGE_FORMAT {
        if *format == *type_file {
            path_dir = PATH_FOR_SAVE_AVATARS;
        }
    }

    // Соответсвует ли формат файла документу
    for format in DOCUMENTS_FORMAT {
        if *format == *type_file {
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
                .join(file_name)
        ).await.ok()  //Возвращает файл или None
}

#[get("/get?<user>&<doc>&<all_users>&<all_docs>")]
async fn get_json_user_doc<'a>(
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
        if let Some(user_v) = user {

            // Получаем HashMap типа <Данные_пользователя, запрашиваемое_значение>
            let hm = user_v.to_hashmap();

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
    }

    // Работа с поиском документов
    if let Some(true) = all_docs { // Если нужны все документы
        response.docs = db.run(
            |conn| db_conn::get_doc(
                HashMap::new(),
                None,
                conn
            )
        ).await.expect("Ошибка при выводе всех документов");

    } else {
        // Если необходимы выбранные документы
        if let Some(doc) = doc {
            let hashmap_doc = doc.to_hashmap();
            let hashmap_author = doc.author_to_hashmap();

            // Если были введены поля для документа ИЛИ для автора документа
            if (hashmap_doc.len() != 0) || (hashmap_author != None) {
                response.docs = db.run(
                    move |conn| db_conn::get_doc(hashmap_doc, hashmap_author, conn)
                ).await.expect("Ошибка при выводе документов по параметрам");
            }
        }
    }
    Json(response)
}




#[post("/add_doc", data= "<file>")]
async fn new_doc(db: Db, mut file: Form<DocumentFile<'_>>) -> Json<String> {

    let filed = file.docfile_to_doc().await;
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
    })
}