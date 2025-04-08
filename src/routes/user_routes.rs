use std::sync::Arc;

use axum::{extract::State, routing::get, Router};

use crate::{db::DbConn, model::AuthenticatedUser, responses::{ApiError, OkResponse}};

async fn get_user_links(user: AuthenticatedUser, State(db): State<Arc<DbConn>>) -> Result<OkResponse<Vec<(String,String)>>, ApiError> {
    let user_id = db.get_user_id(&user.0.sub).unwrap().unwrap();
    match db.get_user_links(user_id) {
        Ok(links) => Ok(OkResponse::new(links)),
        Err(_) => Err(ApiError::InternalServerError),
    }
}


pub fn user_router() -> Router<Arc<DbConn>> {
    Router::new()
        .route("/get-user-links", get(get_user_links))
}