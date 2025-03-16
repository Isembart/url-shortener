use rocket::http::Method;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};

pub fn cors_fairing() -> Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::all(), // Allow all origins
        allowed_methods: vec![Method::Get, Method::Post, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allow_credentials: true, // Needed if using cookies or authentication
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration failed")
}
