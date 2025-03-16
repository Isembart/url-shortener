#[macro_use] extern crate rocket;

mod db;
use std::env;

use db::DbConn;

use md5;
use dotenv::dotenv;
use rocket::fs::FileServer;

use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::State;
use rocket::http::Status;
use rocket::response::status;

mod cors;

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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorResponse {
    error: String,
}

#[post("/shorten-link", data = "<link>")]
fn shorten_link(link: Json<LinkData<'_>>, db: &State<DbConn>) -> Result<Json<ShortLink>, status::Custom<Json<ErrorResponse>>> {
    let short_link: String;

    if let Some(code) = link.code {
        println!("code: {}", code);
        match db.get_long_url(code) {
            Ok(response) => {
                match response {
                    Some(_) => {
                        // there is already a URL with this code, we return error with explanation in JSON
                        return Err(status::Custom(Status::Conflict, Json(ErrorResponse { error: "Jakiś pajac już zapisał link z takim kodem".to_string() })));
                    },
                    None => {
                        short_link = code.to_string();
                    },
                }
            },
            Err(_) => {
                // there is an error with the db connection
                return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse { error: "nie mogę się połączyć z bazą ~bazownik".to_string() })));
            },
        }
    } else {
        short_link = format!("{:x}", md5::compute(link.url))[..6].to_string();
    }

    let long_url = link.url.to_string();

    match db.insert_url(&short_link, &long_url) {
        Ok(_) => Ok(Json(ShortLink { short_url: short_link })),
        Err(_) => Err(status::Custom(Status::InternalServerError, Json(ErrorResponse { error: "Nie udało się zapisać rekordu w bazie".to_string() }))),
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
    dotenv().ok();
    
    let server_port = env::var("URL_SHORTENER_PORT").expect("You must set URL_SHORTENER_PORT in .env file");
    let server_port: u16 = server_port.parse().expect("PORT must be a valid number");

    let server_address = env::var("URL_SHORTENER_ADDRESS").expect("You must set URL_SHORTENER_ADDRESS in .env file");
    let server_address: &str = &server_address;

    let db = DbConn::new("urls.db").expect("Failed to connect to database");
    db.init_db().expect("Failed to initialize database");

    rocket::build()
    .attach(cors::cors_fairing())
    .configure(rocket::Config::figment()
        .merge(("port", server_port))
        .merge(("address", server_address)))
    .mount("/", routes![shorten_link, redirect])
    .mount("/", FileServer::from("./public/www"))
    .manage(db)
}
