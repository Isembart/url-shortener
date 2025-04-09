use std::sync::Arc;

use axum::{extract::{self, State}, response::Redirect, routing::{get, post}, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use tower_cookies::{cookie::{self, time::{Duration, OffsetDateTime}}, Cookie, Cookies};

use crate::{db::{DbConn, UserError}, model::{get_jwt_encoding_key, validate_jwt_token, AuthenticatedUser, Claims}, responses::{ApiError, OkResponse}};


#[derive(Deserialize)]
struct LoginFormData {
    username: String,
    password: String,
    persistent: Option<bool>,
}

async fn refresh(cookies: Cookies) -> Result<OkResponse<String>, ApiError> {
    if let Some(refresh_cookie) = cookies.get("refresh_token") {
        let token = refresh_cookie.value();
        match validate_jwt_token(token) {
            Ok(claims) => {
                let new_claims = Claims{
                    sub: claims.sub.clone(),
                    exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    persistent: claims.persistent,
                };
                let new_token = match encode(&Header::default(), &new_claims, &EncodingKey::from_secret(get_jwt_encoding_key())) {
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
              
                let refresh_token = match encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(get_jwt_encoding_key())) {
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
            Err(_) => Err(ApiError::Forbidden)
        }
    } else {
        Err(ApiError::Forbidden)
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

            let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(get_jwt_encoding_key())) {
                Ok(token) => token,
                Err(_) => {
                    eprintln!("Error generating token");
                    return Err(ApiError::CannotGenerateToken)
                }
            };

            let refresh_claims = Claims {
                exp: match claims.persistent {
                    true => (chrono::Utc::now() + chrono::Duration::days(30)).timestamp() as usize,
                    false => (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                },
                ..claims
            };

            let refresh_token = match encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(get_jwt_encoding_key())) {
                Ok(token) => token,
                Err(_) => {
                    eprintln!("Error generating token");
                    return Err(ApiError::CannotGenerateToken)
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
        Err(UserError::InvalidCredentials) => Err(ApiError::InvalidCredentials),
        Err(UserError::DatabaseError) => Err(ApiError::InternalServerError),
        Err(_) => Err(ApiError::InternalServerError),
    }
}

async fn create_user(State(db): State<Arc<DbConn>>, extract::Json(login_info): extract::Json<LoginFormData>) -> Result<OkResponse<String>, ApiError>{
    match db.create_user(&login_info.username, &login_info.password) {
        Ok(_) => Ok(OkResponse::new(format!("User {} created successfully!", login_info.username))),
        Err(UserError::UserAlreadyExists) => Err(ApiError::UserAlreadyExists),
        Err(_) => Err(ApiError::InternalServerError),
    }
}

async fn whoami(user: AuthenticatedUser) -> Result<OkResponse<Claims>, ApiError>  {
    Ok(OkResponse::new(user.0))
}


pub fn auth_router() -> Router<Arc<DbConn>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/refresh", get(refresh))
        .route("/whoami", get(whoami))
        .route("/create_user", post(create_user))
}