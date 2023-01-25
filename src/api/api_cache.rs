use std::collections::HashMap;
use std::fs;
use std::io::Write;
use rocket::fairing::AdHoc;
use rocket::serde::Serialize;
use rocket::tokio::sync::Mutex;
use serde::Deserialize;

use super::{User, Document};

#[derive(Serialize, Deserialize, Debug)]
struct DeserCache{
    documents: HashMap<String, Document>,
    users: HashMap<String, User>,
}

pub struct ApiCache {
    documents: Mutex<HashMap<String, Document>>,
    users: Mutex<HashMap<String, User>>,
}

impl ApiCache {

    pub fn new() -> Self {

        let file_str = if let Ok(str) = fs::read_to_string("cache_ser.json") {
            str
        } else {
            let default_str = r#"{"documents":{}, "users":{}}"#;
            let mut file = std::fs::File::create("cache_ser.json").unwrap();
            file.write(&default_str.as_bytes()).unwrap();
            default_str.to_owned()
        };
        let data: DeserCache = serde_json::from_str(&file_str).unwrap();

        ApiCache {
            documents: Mutex::new(data.documents),
            users: Mutex::new(data.users),
        }
    }

    pub async fn write_cache_to_json(&self) {
        let cache_string = serde_json::to_string(
            &DeserCache {
                documents: self.documents.lock().await.clone(),
                users: self.users.lock().await.clone(),
            }
        ).unwrap();

        fs::write("cache_ser.json", cache_string).unwrap()
    }

    // Добавить бинарный поиск при поомщи реализайии трейта Ord
    // и сравнения уникльного айди документа
    pub async fn get_docs(&self) -> Vec<Document> {
        self.documents.lock().await.clone().into_values().collect()
    }

    pub async fn get_users(&self) -> Vec<User> {
        self.users.lock().await.clone().into_values().collect()
    }

    pub async fn get_doc_by_uuid(&self, doc_uuid: &str) -> Option<Document> {
        self.documents.lock().await.get(doc_uuid).cloned()
    }

    pub async fn get_user_by_uuid(&self, user_uuid: &str) -> Option<User> {
        self.users.lock().await.get(user_uuid).cloned()
    }

    pub async fn set_docs(&self, docs: &Vec<Document>) {
        for doc in docs {
            self.append_doc(doc.clone()).await;
        }
    }

    pub async fn set_users(&self, users: &Vec<User>) {
        for user in users {
            self.append_user(user.clone()).await;
        }
    }

    pub async fn append_doc(&self, doc: Document) {
        self.documents.lock().await.insert(doc.doc_uuid.clone(), doc);
    }

    pub async fn append_user(&self, user: User) {
        self.users.lock().await.insert(user.uuid.clone(), user);
    }

    pub async fn remove_doc(&self, doc_uuid: &str) -> Option<Document> {
        self.documents.lock().await.remove(doc_uuid)
    }

    pub async fn remove_user(&self, user_uuid: &str) -> Option<User> {
        self.users.lock().await.remove(user_uuid)
    }
}

pub fn state() -> AdHoc {
    AdHoc::on_shutdown("Bye!",|rocket| Box::pin(async move {
        rocket.state::<ApiCache>().unwrap().write_cache_to_json().await;
        println!("Finish write data")
    }))
}