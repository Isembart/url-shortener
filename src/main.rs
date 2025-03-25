#[macro_use] extern crate rocket;

mod db;
mod model;
use std::env;
use db::{DbConn, UserError};
use jsonwebtoken::{encode, EncodingKey, Header};
use md5;
use dotenv::dotenv;
use model::{AuthenticatedUser, Claims, LoginResponse};
use rocket::fs::FileServer;

use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::time::{Duration, OffsetDateTime};
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::status;

mod cors;

#[derive(Deserialize)]
#[serde(crate= "rocket::serde")]
struct LinkData<'r>{
    url: &'r str,
    code: Option<&'r str>,
}

#[derive(Deserialize)]
struct UserData<'r> {
    username: &'r str,
    password: &'r str,
    persistent: Option<bool>,
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
fn shorten_link(user: AuthenticatedUser, link: Json<LinkData<'_>>, db: &State<DbConn>) -> Result<Json<ShortLink>, status::Custom<Json<ErrorResponse>>> {
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

    match db.insert_url(&short_link, &long_url, db.get_user_id(&user.0.sub).unwrap().unwrap()) {
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

#[post("/create-user", data = "<user>")]
fn create_user(user: Json<UserData<'_>>, db: &State<DbConn>) -> Result<Json<ShortLink>, status::Custom<Json<ErrorResponse>>> {
 
    match db.create_user(user.username, user.password) {
        Ok(_) => Ok(Json(ShortLink { short_url: user.username.to_string() })),
        Err(UserError::UserAlreadyExists) => Err(status::Custom(Status::Conflict, Json(ErrorResponse {error:"Użytkownik już istnieje".to_string()}))),
        Err(_) => Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się utworzyć użytkownika".to_string()}))),
    }
}

#[post("/login", data = "<login_info>")]
fn login(jar: &CookieJar<'_>, login_info: Json<UserData<'_>>, db: &State<DbConn>) -> Result<Json<LoginResponse>, status::Custom<Json<ErrorResponse>>> {
    match db.login(login_info.username, login_info.password) {
        Ok(_) => {
            let claims = Claims{
                sub: login_info.username.to_string().clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                persistent: login_info.persistent.unwrap_or(false),
            };
           
            let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                Ok(token) => token,
                Err(_) => {
                    eprintln!("Error generating token");
                    return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się wygenerować tokena".to_string()})));
                }
            };
 
            let refresh_claims = Claims {
                exp: match claims.persistent {
                    true => (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
                    false => (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                },
                ..claims
            };

            let refresh_token = match encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                Ok(token) => token,
                Err(_) => {
                    eprintln!("Error generating token");
                    return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się wygenerować tokena".to_string()})));
                }
            };

            let refresh_token_duration = match claims.persistent {
                true => Duration::days(30),
                false => Duration::hours(1),
            };
            //send the refresh token in a http-only cookie
            jar.add(Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(true)
                .expires(OffsetDateTime::now_utc() + refresh_token_duration)
                .max_age(refresh_token_duration)
                .same_site(rocket::http::SameSite::Lax));
            
            // Ok(Json(ShortLink { short_url: user.username.to_string() }))
            Ok(Json(LoginResponse{token}))
        },
        Err(UserError::InvalidCredentials) => Err(status::Custom(Status::Unauthorized, Json(ErrorResponse {error:"Nieprawidłowe dane logowania".to_string()}))),
        Err(UserError::DatabaseError(err)) => Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error: err.to_string()}))),
        Err(_) => Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Mowiąc kolokwialnie, coś się rozjebało".to_string()}))),
    }
}

#[get("/refresh")]
fn refresh(jar: &CookieJar<'_>) -> Result<Json<LoginResponse>, status::Custom<Json<ErrorResponse>>> {
    if let Some(cookie) = jar.get("refresh_token") {
        let token = cookie.value();
        match model::validate_jwt_token(token) {
            Ok(claims) => {
                let new_claims = Claims{
                    sub: claims.sub.clone(),
                    exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    persistent: claims.persistent,
                };
                let new_token = match encode(&Header::default(), &new_claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                    Ok(token) => token,
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się wygenerować tokena".to_string()}))),
                };

                //before returning the new token, also make a new refresh token
                let refresh_claims = Claims {
                    exp: match claims.persistent {
                        true => (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
                        false => (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    },
                    ..new_claims
                };
              
                let refresh_token = match encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                    Ok(token) => token,
                    Err(_) => {
                        eprintln!("Error generating token");
                        return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się wygenerować tokena".to_string()})));
                    }
                };

                let refresh_token_duration = match claims.persistent {
                    true => Duration::days(30),
                    false => Duration::hours(1),
                };
                //send the refresh token in a http-only cookie
                jar.add(Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(true)
                .expires(OffsetDateTime::now_utc() + refresh_token_duration)
                .max_age(refresh_token_duration)
                .same_site(rocket::http::SameSite::Lax));

                Ok(Json(LoginResponse{token: new_token}))
            },
            Err(_) => Err(status::Custom(Status::Unauthorized, Json(ErrorResponse {error:"Nieprawidłowy token".to_string()}))),
        }
    } else {
        Err(status::Custom(Status::Unauthorized, Json(ErrorResponse {error:"Brak tokena".to_string()})))
    }


}

#[get("/logout")]
fn logout(user: AuthenticatedUser, jar: &CookieJar<'_>) -> Result<Json<ErrorResponse>, status::Custom<Json<ErrorResponse>>> {
    println!("User {} logged out", user.0.sub);
    if jar.get("refresh_token").is_some() {
        jar.remove("refresh_token");
    }
    
    Ok(Json(ErrorResponse {error:"Zostałeś wylogowany".to_string()}))
}

#[get("/whoami")]
fn whoami(user: AuthenticatedUser) -> Json<Claims> {
    Json(user.0)
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
    .mount("/", routes![shorten_link, redirect, create_user, login, refresh, whoami, logout])
    .mount("/", FileServer::from("./public/www"))
    .manage(db)
}
