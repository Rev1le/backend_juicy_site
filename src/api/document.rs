use std::collections::HashMap;
use rocket::{
    fs::TempFile,
    serde::{Deserialize, Serialize}
};

use super::user::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FromForm)]
pub struct Document {
    pub title: String,
    pub path: String,
    pub author: User,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
    pub doc_uuid: String,
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

// Переписать добавление файла Пользователь с пустыми ячейками - говно
impl<'a> DocumentFile<'a> {
    pub async fn docfile_to_doc(&mut self, path_to_save_docs: &str) -> Document {
        use uuid::Uuid;

        let doc_uuid = Uuid::new_v4().to_string();
        let file_name = format!("{}.{}", doc_uuid, self.file_type);
        let path = format!("{}{}", path_to_save_docs, file_name);
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
            doc_uuid
        }
    }
}

//Структура для запроса документа
#[derive(Debug, FromForm)]
pub struct DocumentFromRequest<'a> {
    title: Option<&'a str>,
    path: Option<&'a str>,
    author: UserFromRequest<'a>,
    subject: Option<&'a str>,
    type_work: Option<&'a str>,
    number_work: Option<&'a str>,
    doc_uuid: Option<&'a str>,
}

impl<'a> DocumentFromRequest<'a> {

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut user_hm_params: HashMap<String, String> = HashMap::new();

        if let Some(title) = self.title {
            user_hm_params.insert(
                "title".to_string(),
                title.to_string()
            );
        }
        if let Some(path) = self.path {
            user_hm_params.insert(
                "path".to_string(),
                path.to_string()
            );
        }
        if let Some(subject) = self.subject {
            user_hm_params.insert(
                "subject".to_string(),
                subject.to_string()
            );
        }
        if let Some(type_work) = self.type_work {
            user_hm_params.insert(
                "type_work".to_string(),
                type_work.to_string()
            );
        }
        if let Some(number_work) = self.number_work {
            user_hm_params.insert(
                "number_work".to_string(),
                number_work.to_string()
            );
        }
        if let Some(doc_uuid) = self.doc_uuid {
            user_hm_params.insert(
                "doc_uuid".to_string(),
                doc_uuid.to_string()
            );
        }
        user_hm_params
    }

    pub fn get_author(&self) -> UserFromRequest {
        self.author
    }

    pub fn author_to_hashmap(&self) -> HashMap<String, String> {
        self.author.to_hashmap()
    }
}