use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{extract, Router};
use serde::Deserialize;
use crate::DbConn;
use crate::model::AuthenticatedUser;
use crate::responses::{ApiError, OkResponse};

#[derive(Deserialize)]
struct LinkData {
    url: String,
    code: Option<String>
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

pub fn url_shortener_router() -> Router<Arc<DbConn>> {
    Router::new()
        .route("/shorten", post(shorten_link))
        .route("/link/{short_url}", get(redirect))
}