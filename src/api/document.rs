
use super::user::{
    User,
    UserFromRequest
};

use std::collections::HashMap;
use rocket::{
    fs::TempFile,
    serde::{
        Deserialize,
        Serialize
    }
};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, FromForm)]
#[serde(crate = "rocket::serde")]
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

impl<'a> DocumentFile<'a> {
    pub async fn docfile_to_doc(&mut self) -> Document {
        use uuid::Uuid;
        use crate::api::PATH_FOR_SAVE_DOCS;

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
            doc_uuid
        }
    }
}

//Структура для запроса документа
#[derive(Debug, FromForm)]
pub struct DocumentFromRequest<'a> {
    title: Option<&'a str>,
    path: Option<&'a str>,
    author: Option<UserFromRequest<'a>>,
    subject: Option<&'a str>,
    type_work: Option<&'a str>,
    number_work: Option<&'a str>,
    doc_uuid: Option<&'a str>,
}

impl<'a> DocumentFromRequest<'a> {

    pub fn check_doc(&self)
                 -> (
                     HashMap<String, String>, // Для полей документа
                     Option<HashMap<String, String>> // Для полей автора
                 ) {
        let mut res: (
            HashMap<String, String>,
            Option<HashMap<String, String>>) = (HashMap::new(), None);

        if let Some(title) = self.title {
            res.0.insert(
                "title".to_string(),
                title.to_string()
            );
        }
        if let Some(path) = self.path {
            res.0.insert(
                "path".to_string(),
                path.to_string()
            );
        }
        if let Some(author) = self.author {
            let tmp = author.check_user();
            if tmp.len() != 0 {
                res.1 = Some(tmp);
            }
        }
        if let Some(subject) = self.subject {
            res.0.insert(
                "subject".to_string(),
                subject.to_string()
            );
        }
        if let Some(type_work) = self.type_work {
            res.0.insert(
                "type_work".to_string(),
                type_work.to_string()
            );
        }
        if let Some(number_work) = self.number_work {
            res.0.insert(
                "number_work".to_string(),
                number_work.to_string()
            );
        }
        if let Some(doc_uuid) = self.doc_uuid {
            res.0.insert(
                "doc_uuid".to_string(),
                doc_uuid.to_string()
            );
        }
        res
    }
}