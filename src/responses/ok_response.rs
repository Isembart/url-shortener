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