mod api;
mod auth;

/// Иконка сайта
#[get("/favicon.ico")] //Иконка сайта
async fn icon() -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("icon_site.ico").await.ok()
}

/// Главная страница сайта
#[get("/")]
async fn index() -> rocket::serde::json::Json<bool> {
    rocket::serde::json::Json(true)
}

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                icon
            ]
        )
        .attach(api::stage())
        .attach(auth::stage())
}
