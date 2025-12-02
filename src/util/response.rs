use actix_web::{HttpResponse, Responder};
use serde::Serialize;
use std::io::Error;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            code: 200,
            msg: "ok".to_string(),
            data: Some(data),
        }
    }

    fn fail(error: Error) -> Self {
        Self {
            code: -1,
            msg: error.to_string(),
            data: None,
        }
    }

    pub fn response(res: Result<T, Error>) -> Self {
        match res {
            Ok(data) => Self::success(data),
            Err(e) => Self::fail(e),
        }
    }
}

// 实现 Responder trait，使其可以直接在 Actix-Web 中返回
impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}
