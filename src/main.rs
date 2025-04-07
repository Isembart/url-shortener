mod responses;
mod routes;
mod model;
mod db;
use routes::routes;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use axum::Router;
use db::DbConn;
use tower_http::services::ServeDir;

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

    // Resolve the address to support both IPv4 and IPv6
    let listener_address = format!("{}:{}", server_address, server_port).to_socket_addrs()
        .expect("Invalid server address")
        .next()
        .expect("Could not resolve address");

    let main_router: Router = Router::new()
        .fallback_service(ServeDir::new("./public/www"))
        .route("/test", axum::routing::get(|| async { "Hello, world!" }))
        .merge(routes())
        .layer(CookieManagerLayer::new())
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(db)
        ;

    println!("Started server at http://{}", &listener_address);

    let listener = tokio::net::TcpListener::bind(&listener_address).await.unwrap();
    axum::serve(listener, main_router).await.unwrap();
}