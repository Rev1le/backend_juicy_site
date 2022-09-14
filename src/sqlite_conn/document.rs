use std::fmt;
use rocket::serde::{
    Serialize,
    Deserialize
};

use crate::sqlite_conn::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub title: String,
    pub path: String,
    pub author: User,
    pub subject: String,
    pub type_work: String,
    pub number_work: i64,
    pub note: Option<String>,
}

impl Document {
    pub fn new(
        title: String,
        path: String,
        author: User,
        subject: String,
        type_work: String,
        number_work: i64,
        note: Option<String>,
    ) -> Document {
        Document {
            title,
            path,
            author,
            subject,
            type_work,
            number_work,
            note,
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "||Сведения об документе||\nДокумент: {}\nРасположение: {}\nАвтор: {}\nПредмет: {}\n\
            Тип работы: {}\nНомер работы: {}\nПримечание: {:?}\n",
            self.title, self.path, self.author, self.subject, self.type_work,
            self.number_work, self.note
        )
    }
    /*
    fn vector_to_struct(vec: &Vec<String>) -> Self {
        Document::new_user(
            vec[0].clone(),
            PathBuf::from(vec[2].clone()),
            vec[0].clone(),
            vec[3].clone(),
            FromStr::from_str(vec[4].clone().as_str()).unwrap(),
            FromStr::from_str(vec[5].clone().as_str()).unwrap(),
            vec[6].clone()
        )
    }

     */
}