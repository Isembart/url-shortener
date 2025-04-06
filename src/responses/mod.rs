use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use chrono;

#[derive(Serialize, Debug)]
pub struct OkResponse<T: Serialize> {
    pub data: T,
    pub timestamp: String,
}

impl<T:Serialize> OkResponse<T> {
    pub fn new(response_data: T) -> OkResponse<T>{
        OkResponse{ data: response_data, timestamp: chrono::Utc::now().to_rfc3339() }
    }
}

impl<T: Serialize> IntoResponse for OkResponse<T>{

    fn into_response(self) -> Response {
        let status = StatusCode::OK;
        let body = Json(self);
        (status,body).into_response()
    }
}

// impl<T:Serialize> IntoResponse for Result<OkResponse<T>, ApiError> {
//     fn into_response(self) -> Response {
//         match self {
//             Ok(ok_response) => ok_response.into_response(),
//             Err(api_error) => api_error.into_response(),
//         }    
//     }
// }

#[derive(Serialize)]
pub struct ErrResponseBody {
    pub error: String,
    pub timestamp: String,
}

#[derive(Serialize)]
#[derive(Clone)]
pub enum ApiError{
    AuthError,
    Forbidden,
    NotFound,
    CannotGenerateToken,
    InvalidCredentials,
    InternalServerError,
    UserAlreadyExists,
    Conflict,


}

impl ApiError{
    pub fn message(&self) -> &'static str {
        match self {
            ApiError::AuthError => "User not authenticated",
            ApiError::NotFound => "Data not found",
            ApiError::CannotGenerateToken => "Could not generate access token",
            ApiError::InvalidCredentials => "Invalid Credentials",
            ApiError::InternalServerError => "Internal server error",
            ApiError::UserAlreadyExists => "User already exists",
            ApiError::Conflict => "Data already exists",
            ApiError::Forbidden => "Forbidden",


        }
    }

    pub fn to_response_body(&self) -> ErrResponseBody {
        ErrResponseBody { error: self.message().to_string(), timestamp: chrono::Utc::now().to_rfc3339() }
    }

    fn status_code(&self) -> StatusCode{
        match self {
            ApiError::AuthError => StatusCode::UNAUTHORIZED,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::CannotGenerateToken => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UserAlreadyExists => StatusCode::CONFLICT,
            ApiError::Conflict => StatusCode::CONFLICT,
            ApiError::Forbidden => StatusCode::FORBIDDEN,            


        }
    }
}

impl IntoResponse for ApiError{
    fn into_response(self) -> Response{
        let status = self.status_code();
        let body = Json(self.to_response_body());
        (status,body).into_response()
    }
}


