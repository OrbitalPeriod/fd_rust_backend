use actix_web::{http::StatusCode, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub status_code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status_code).unwrap()).json(self)
    }
}

impl ApiResponse<()> {
    pub fn new_internal_error<T: Serialize>(message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            status_code: 500,
            message: message.into(),
            data: None,
        }
    }
    pub fn new_not_found_error<T: Serialize>(message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            status_code: 404,
            message: message.into(),
            data: None,
        }
    }
    pub fn new_ok<T: Serialize>(message: impl Into<String>, data: T) -> ApiResponse<T> {
        ApiResponse {
            status_code: 200,
            message: message.into(),
            data: Some(data),
        }
    }
    pub fn new_ok_no_data<T: Serialize>(message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            status_code: 200,
            message: message.into(),
            data: None,
        }
    }
    pub fn new_bad_request<T: Serialize>(message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            status_code: 400,
            message: message.into(),
            data: None,
        }
    }
    pub fn new_no_data_found<T: Serialize>(message: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            status_code: 456,
            message: message.into(),
            data: None,
        }
    }
}
