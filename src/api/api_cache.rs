use std::collections::HashMap;
use rocket::http::ext::IntoCollection;
use rocket::tokio::sync::Mutex;

use super::{User, Document};

pub struct ApiCache {
    documents: Mutex<HashMap<String, Document>>,
    users: Mutex<HashMap<String, User>>,
}

impl ApiCache {

    pub fn new() -> Self {
        ApiCache {
            documents: Mutex::new(HashMap::default()),
            users: Mutex::new(HashMap::default()),
        }
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