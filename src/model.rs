
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rand::{thread_rng, RngCore};
use rocket::{http::Status, request::{FromRequest, Outcome}, Request};
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
pub struct AuthError();

impl rocket::response::Responder<'_, 'static> for AuthError {
    fn respond_to(self, _: &'_ rocket::Request) -> rocket::response::Result<'static> {
        Err(Status::Unauthorized)
    }
}

#[derive(Debug)]
pub struct AuthenticatedUser(pub Claims);


#[async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // Extract the "Authorization" header
        if let Some(auth_header) = req.headers().get_one("Authorization") {
            match validate_jwt_token(auth_header) {
                Ok(claims) => Outcome::Success(AuthenticatedUser(claims)),
                Err(e) => Outcome::Error((Status::Forbidden, e)),
            }
        } else {
            Outcome::Error((Status::Forbidden, AuthError()))
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


pub fn validate_jwt_token(token: &str) -> Result<Claims, AuthError> {
    let decoding_key = DecodingKey::from_secret(get_jwt_encoding_key()); 
    
    let validation = Validation::new(Algorithm::HS256);
    
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err(AuthError()),
    }
}