use rocket::{Rocket, Build};
use rocket::fairing::AdHoc;
use rocket::form::Form;

use reqwest;

#[derive(FromForm)]
struct AuthUser {
    login: String
}

#[post("/", data="<data>")]
async fn index(data: Form<AuthUser>) -> &'static str {
    reqwest::get("127.0.0.1:8090/");
    return "Hello Authenifacion"
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite(
        "Auth stage",
        |rocket| async {
            rocket.mount("/auth", routes![index])
        }
    )
}