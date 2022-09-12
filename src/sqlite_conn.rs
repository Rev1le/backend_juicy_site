pub mod user;
pub mod document;

use std::{{path::PathBuf, str::FromStr}, collections::HashMap, fmt};
use rocket::serde::json::Json;
use user::{User, UserEvent};
use document::Document;
use sqlite;

pub struct DataBase {
    pub connection: sqlite::Connection,
    pub address: String,
}


impl DataBase {
    pub fn new(path: &str) -> Self {
        let connection = sqlite::open(path).unwrap();
        connection.execute(
            "CREATE TABLE IF NOT EXISTS users (
            name TEXT,
            nickname TEXT,
            avatar TEXT,
            role TEXT,
            admin TEXT,
            tg_id INTEGER,
            uuid TEXT );"
        ).unwrap();
        connection.execute(
            "CREATE TABLE IF NOT EXISTS documents (
            title TEXT,
            path TEXT,
            author_uuid TEXT,
            subject TEXT,
            type_work TEXT,
            number_work INTEGER,
            note TEXT);"
        ).unwrap();
        DataBase{
            connection,
            address: path.clone().to_string()
        }
    }

    pub fn get_all_user<'a, 'b>(&self) -> Vec<User> {
        let mut vec_users: Vec<Vec<String>>  = Vec::new(); // записываем вектора, которые содержат поля объекта User
        //let execute_str: String = format!("SELECT * FROM users WHERE {} = '{}'", pa);
        self.connection
            .iterate("SELECT * FROM users", |pairs| {
                let mut user_vec_field: Vec<String> = Vec::new();
                for &(column, value) in pairs.iter() {
                    user_vec_field.push(value.unwrap().to_string());
                }

                vec_users.push(user_vec_field);
                true
            }).unwrap();


        let mut vec_user: Vec<User> = Vec::new();

        for user_tmp in vec_users {
            let user = User::vector_to_struct(&user_tmp);
            vec_user.push(user);
        }
        vec_user
    }

    pub fn get_user(&self, dict: HashMap<&str, &str>) -> Option<Vec<User>>{
        let mut execute_str = format!("SELECT * FROM users WHERE ");
        for (column, value) in dict {
            let vec_ch = value.chars();
            let mut new_value_str = String::new();
            for v in vec_ch { //Отслеживание SQl инъекций
                match v {
                    '\'' => {continue}//println!("Одинар"),
                    '\"' => {continue}//println!("Двойная"),
                    _ => new_value_str.push(v)
                }
            }
            //println!("{}", new_value_str);

            let tmp = format!("{column} = '{new_value_str}' AND ");
            execute_str += &tmp;
        }
        let res = &execute_str[0..execute_str.len()-4]; // Послоедние слова всегдла будут AND
        //println!("{:?}", res);
        let mut vec_users: Vec<Vec<String>>  = Vec::new();
        self.connection
            .iterate(res, |pairs| {
                let mut user_vec_field: Vec<String> = Vec::new();
                for &(column, value) in pairs.iter() {
                    user_vec_field.push(value.unwrap().to_string());
                }

                vec_users.push(user_vec_field);
                true
            }).unwrap();


        let mut vec_user: Vec<User> = Vec::new();

        for user_tmp in vec_users {
            let user = User::new_user(
                user_tmp[0].clone(),
                user_tmp[1].clone(),
                PathBuf::from(user_tmp[2].clone()),
                user_tmp[3].clone(),
                FromStr::from_str(user_tmp[4].clone().as_str()).unwrap(),
                FromStr::from_str(user_tmp[5].clone().as_str()).unwrap(),
                user_tmp[6].clone()
            );
            vec_user.push(user);
        }
        if vec_user.len() == 0 {
            return None
        }
        Some(vec_user)
    }

    pub fn add_doc(&self, doc: Document) {
        let execute_str = format!("INSERT INTO documents VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{:?}')",
                                  doc.title,
                                  doc.path,
                                  doc.author.uuid,
                                  doc.subject,
                                  doc.type_work,
                                  doc.number_work,
                                  doc.note);
        self.connection.execute(execute_str).unwrap()
    }

    pub fn get_doc(&self, dict: HashMap<&str, &str>) -> Option<Vec<Document>>{
        //Добавить дополнения для автора
        let mut execute_str = format!("SELECT * FROM documents WHERE ");
        for (column, value) in dict { //защита от SQl инъекций
            let vec_ch = value.chars();
            let mut new_value_str = String::new();
            for v in vec_ch { //Отслеживание SQl инъекций
                match v {
                    '\'' => {continue}//println!("Одинар"),
                    '\"' => {continue}//println!("Двойная"),
                    _ => new_value_str.push(v)
                }
            }
            //println!("{}", new_value_str);

            let tmp = format!("{column} = '{new_value_str}' AND ");
            execute_str += &tmp;
        }
        let res = &execute_str[0..execute_str.len()-4]; // Послоедние слова всегдла будут AND
        //println!("{:?}", res);
        let mut vec_docs: Vec<Vec<String>>  = Vec::new();
        self.connection
            .iterate(res, |pairs| {
                let mut doc_vec_field: Vec<String> = Vec::new();
                for &(column, value) in pairs.iter() {
                    doc_vec_field.push(value.unwrap().to_string());
                }

                vec_docs.push(doc_vec_field);
                true
            }).unwrap();
        //println!("{:?}", vec_docs);

        let mut vec_doc: Vec<Document> = Vec::new();

        for doc_tmp in vec_docs {
            let mut doc_user = self.get_user(HashMap::from([("uuid", doc_tmp[2].as_str())]));
            let doc_user_some;
            if doc_user == None {
                println!("\nАвтора документа не удалой найти по uuid\n");
                continue
            } else {
                doc_user_some = doc_user.unwrap()[0].clone();
            }

            let doc = Document::new(
                doc_tmp[0].clone(),
                doc_tmp[1].clone(),
                doc_user_some,
                doc_tmp[3].clone(),
                doc_tmp[4].clone(),
                FromStr::from_str(doc_tmp[5].clone().as_str()).unwrap(),
                Some(doc_tmp[6].clone()),
            );
            vec_doc.push(doc);
        }
        if vec_doc.len() == 0 {
            return None
        }
        return Some(vec_doc)
    }
}
