mod url_shortener_routes;
mod auth_routes;
mod user_routes;

use axum::Router;
use user_routes::user_router;
use std::sync::Arc;
use crate::db::DbConn;
use url_shortener_routes::url_shortener_router;
use auth_routes::auth_router;

pub fn routes() -> axum::Router<Arc<DbConn>> {
    Router::new()
        .merge(url_shortener_router()) 
        .merge(auth_router())
        .merge(user_router())
}