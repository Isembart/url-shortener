// #[macro_use] extern crate rocket;

mod responses;
mod db;
mod model;
use axum::extract;
use axum::extract::State;
use axum::response::Redirect;
use model::{AuthenticatedUser, Claims};
use responses::ApiError;
use responses::OkResponse;
use tower_cookies::cookie;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use axum::Router;
use db::{DbConn, UserError};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tower_cookies::{Cookie, cookie::time::Duration, cookie::time::OffsetDateTime, Cookies};

#[derive(Deserialize)]
struct LinkData {
    url: String,
    code: Option<String>
}

#[derive(Deserialize)]
struct LoginFormData {
    username: String,
    password: String,
    persistent: Option<bool>,
}


async fn shorten_link(State(db): State<Arc<DbConn>>,user: AuthenticatedUser, extract::Json(link): extract::Json<LinkData>) -> Result<OkResponse<String>, ApiError> {
    let short_link: String;
    
    if let Some(code) = link.code {
        println!("code: {}", code);
        match db.get_long_url(code.clone()) {
            Some(_) => {
                // there is already a URL with this code, we return error with explanation in JSON
                // return Err(status::Custom(Status::Conflict, Json(ErrorResponse { error: "Jakiś pajac już zapisał link z takim kodem".to_string() })));
                return Err(ApiError::Conflict)
            },
            None => {
                short_link = code.to_string();
            },
        }
    } else {
        short_link = format!("{:x}", md5::compute(&link.url))[..6].to_string();
    }

    let long_url = link.url.to_string();

    match db.insert_url(&short_link, &long_url, db.get_user_id(&user.0.sub).unwrap().unwrap()) {
        // Ok(_) => Ok(Json(ShortLink { short_url: short_link })),
        Ok(_) => Ok(OkResponse::new(short_link)),
        Err(_) => Err(ApiError::InternalServerError)
        // Err(_) => Err(status::Custom(Status::InternalServerError, Json(ErrorResponse { error: "Nie udało się zapisać rekordu w bazie".to_string() }))),
    }
}


async fn redirect(State(db): State<Arc<DbConn>>, extract::Path(short_url): extract::Path<String>) -> axum::response::Redirect {
    match db.get_long_url(short_url) {
        Some(long_rl) => {
            axum::response::Redirect::permanent(&long_rl)
        },
        None => {
            axum::response::Redirect::permanent("/")
        }
    }
}

async fn create_user(State(db): State<Arc<DbConn>>, extract::Json(login_info): extract::Json<LoginFormData>) -> Result<OkResponse<String>, ApiError>{
    match db.create_user(&login_info.username, &login_info.password) {
        Ok(_) => Ok(OkResponse::new(format!("User {} created successfully!", login_info.username))),
        Err(UserError::UserAlreadyExists) => Err(ApiError::UserAlreadyExists),
        Err(_) => Err(ApiError::InternalServerError),
    }
}


async fn refresh(cookies: Cookies) -> Result<OkResponse<String>, ApiError> {
    if let Some(refresh_cookie) = cookies.get("refresh_token") {
        let token = refresh_cookie.value();
        match model::validate_jwt_token(token) {
            Ok(claims) => {
                let new_claims = Claims{
                    sub: claims.sub.clone(),
                    exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    persistent: claims.persistent,
                };
                let new_token = match encode(&Header::default(), &new_claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                    Ok(token) => token,
                    // Err(_) => return Err(status::Custom(Status::InternalServerError, Json(ErrorResponse {error:"Nie udało się wygenerować tokena".to_string()}))),
                    Err(_) => { return Err(ApiError::InternalServerError) }
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
                    Err(_) => { return Err(ApiError::InternalServerError) }
                };

                let refresh_token_duration = match claims.persistent {
                    true => Duration::days(30),
                    false => Duration::hours(1),
                };

                //send the refresh token in a http-only cookie
                cookies.add(Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(true)
                .expires(OffsetDateTime::now_utc() + refresh_token_duration)
                .max_age(refresh_token_duration)
                .same_site(cookie::SameSite::Lax)
                .build()
                );

                Ok(OkResponse::new(new_token))
            },
            // Err(_) => Err(status::Custom(Status::Unauthorized, Json(ErrorResponse {error:"Nieprawidłowy token".to_string()}))),
            Err(_) => Err(ApiError::AuthError)
        }
    } else {
        Err(ApiError::AuthError)
        // Err(status::Custom(Status::Unauthorized, Json(ErrorResponse {error:"Brak tokena".to_string()})))
    }
}

async fn logout(user: AuthenticatedUser, cookies: Cookies) -> Result<OkResponse<String>, Redirect>{
    println!("{}",user.0.sub);
    let refresh_cookie = cookies.get("refresh_token");
    if refresh_cookie.is_some() {
        cookies.remove(Cookie::build("refresh_token").build()); // Clone so it owns the cookie
        Ok(OkResponse::new("Logged out".to_string()))
    } else {
        Err(Redirect::to("/"))
    }
}


async fn login(State(db): State<Arc<DbConn>>, cookies: Cookies, extract::Json(login_info): extract::Json<LoginFormData> ) -> Result<OkResponse<String>, ApiError> {
    // match Ok(true) {
    match db.login(&login_info.username, &login_info.password) {
        Ok(_) => {
            let claims = Claims {
                sub: login_info.username.to_string().clone(),
                exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                persistent: login_info.persistent.unwrap_or(false),
            };

            let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(model::get_jwt_encoding_key())) {
                Ok(token) => token,
                Err(_) => {
                    eprintln!("Error generating token");
                    return Err(responses::ApiError::CannotGenerateToken)
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
                    return Err(responses::ApiError::CannotGenerateToken)
                }
            };
            let refresh_token_duration = match claims.persistent {
                true => Duration::days(30),
                false => Duration::hours(1),
            };

            cookies.add(Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(true)
                .expires(OffsetDateTime::now_utc() + refresh_token_duration)
                .max_age(refresh_token_duration)
                .same_site(tower_cookies::cookie::SameSite::Lax)
                .build()
            );

            Ok(OkResponse::new(token))
        },
        Err(UserError::InvalidCredentials) => Err(responses::ApiError::InvalidCredentials),
        Err(UserError::DatabaseError(_)) => Err(responses::ApiError::InternalServerError),
        Err(_) => Err(responses::ApiError::InternalServerError),
    }
}

async fn whoami(user: AuthenticatedUser) -> Result<OkResponse<Claims>, ApiError>  {
    Ok(OkResponse::new(user.0))
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let db = Arc::new(DbConn::new("urls.db").expect("Failed to connect to database"));
    db.init_db().expect("Failed to initialize database");

    let server_port: u16 = std::env::var("URL_SHORTENER_PORT")
        .expect("You must set URL_SHORTENER_PORT in .env file")
        .parse()
        .expect("PORT must be a valid number");

    let server_address = std::env::var("URL_SHORTENER_ADDRESS")
        .expect("You must set URL_SHORTENER_ADDRESS in .env file");

    let listener_address: std::net::SocketAddr = format!("{}:{}", server_address, server_port)
        .parse()
        .expect("Invalid server address");



    let main_router: Router = Router::new()
        .fallback_service(ServeDir::new("./public/www"))
        .route("/test", axum::routing::get(|| async {"Hello, world!"}))
        .route("/login", axum::routing::post(login))
        .route("/shorten-link", axum::routing::post(shorten_link))
        .route("/whoami", axum::routing::get(whoami))
        .route("/logout", axum::routing::get(logout))
        .route("/refresh", axum::routing::get(refresh))
        .route("/create-user", axum::routing::post(create_user))
        .route("/link/{short_url}", axum::routing::get(redirect))
        .layer(CookieManagerLayer::new())
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(db)
        ;

    println!("Started server at, http://{}",&listener_address);

    let listener = tokio::net::TcpListener::bind(&listener_address).await.unwrap();
    axum::serve(listener, main_router).await.unwrap();
}