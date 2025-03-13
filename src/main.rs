#[macro_use] extern crate rocket;

mod db;
use db::DbConn;

use md5;
use rocket::fs::FileServer;

use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::form::{Form, FromForm};
use rocket::State;

#[derive(FromForm)]
#[derive(Debug)]
struct LinkForm<'r> {
    url: &'r str,
    code: Option<&'r str>,
}

#[derive(Deserialize)]
#[serde(crate= "rocket::serde")]
struct LinkData<'r>{
    url: &'r str,
    code: Option<&'r str>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ShortLink {
    short_url: String,
}


#[post("/shorten-link", data = "<link>")]
// fn shorten_link(link: Form<LinkForm<'_>>, db: &State<DbConn>) -> Json<ShortLink> {
fn shorten_link(link: Json<LinkData<'_>>, db: &State<DbConn>) -> Json<ShortLink> {
    let short_link = format!("{:x}", md5::compute(link.url));
    let long_url = link.url.to_string();

    match db.insert_url(&short_link, &long_url) {
        Ok(_) => {
            Json(ShortLink { short_url: short_link })
        },
        Err(_) => {
            Json(ShortLink { short_url: "Error".to_string() })
        },
    }
}

#[get("/link/<short_url>")]
fn redirect(short_url: String, db: &State<DbConn>) -> Option<Redirect> {
    match db.get_long_url(&short_url) {
        Ok(long_url) => long_url.map(|url| Redirect::to(url)),
        Err(_) => None
    }
}

#[launch]
fn rocket() -> _ {
    let db = DbConn::new("urls.db").expect("Failed to connect to database");
    db.init_db().expect("Failed to initialize database");

    rocket::build()
    .mount("/", routes![shorten_link, redirect])
    .mount("/", FileServer::from("./public/www"))
    .manage(db)
}
