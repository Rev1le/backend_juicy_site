
use crate::user_request::UserFromRequest;

use std::collections::HashMap;

//Структура для запроса документа
#[derive(Debug,  FromForm)]
pub(crate) struct DocumentFromRequest<'a> {
    title: Option<&'a str>,
    path: Option<&'a str>,
    author: Option<UserFromRequest<'a>>,
    subject: Option<&'a str>,
    type_work: Option<&'a str>,
    number_work: Option<&'a str>,
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
        res
    }
}