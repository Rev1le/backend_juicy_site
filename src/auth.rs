use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;


#[get("/")]
async fn index() -> &'static str {
    return "Hello"
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Auth stage", |rocket| async {
        rocket.mount("/auth", routes![index])
    })
}