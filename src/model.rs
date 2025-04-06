
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::responses::ApiError;

use super::responses::ApiError::AuthError;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginResponse{
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims{
    pub sub: String,
    pub exp: usize,
    pub persistent: bool,
}


#[derive(Debug)]
pub struct AuthenticatedUser(pub Claims);


// #[async_trait]
// impl<'r> FromRequest<'r> for AuthenticatedUser {
impl<S> FromRequestParts<S> for AuthenticatedUser
where S: Send + Sync
{
    type Rejection = ApiError;
    // type Error = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the "Authorization" header
        if let Some(auth_header) = parts.headers.get("Authorization") {
            match auth_header.to_str() {
                Ok(auth_str) => match validate_jwt_token(auth_str) {
                    Ok(claims) => Ok(AuthenticatedUser(claims)),
                    Err(_) => Err(AuthError),
                },
                Err(_) => Err(AuthError),
            }
        } else {
            // Outcome::Error((Status::Forbidden, AuthError()))
            Err(AuthError)
        }
    }
}




use std::sync::OnceLock;

// Create a static global key
static JWT_ENCODING_KEY: OnceLock<[u8; 32]> = OnceLock::new();

fn generate_secret_key() -> [u8; 32] {
    let mut key = [0u8; 32]; // 256-bit key for HS256
    thread_rng().fill_bytes(&mut key);
    key
}

// Function to get the key (it will be initialized once)
pub fn get_jwt_encoding_key() -> &'static [u8; 32] {
    JWT_ENCODING_KEY.get_or_init(|| { generate_secret_key() } )
}


pub fn validate_jwt_token(token: &str) -> Result<Claims, ApiError> {
    let decoding_key = DecodingKey::from_secret(get_jwt_encoding_key()); 
    
    let validation = Validation::new(Algorithm::HS256);
    
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err(AuthError),
    }
}