use crate::api::req::file::DeleteFileReq;
use crate::app::conf::AppState;
use crate::service;
use crate::util::response::ApiResponse;
use crate::util::upload::UploadFileResp;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, get, post, web};

// 上传请求

// 图片上传
#[post("/upload/image")]
pub async fn upload_image(
    app_state: web::Data<AppState>,
    payload: Multipart,
) -> ApiResponse<UploadFileResp> {
    ApiResponse::response(service::file::upload_image(&app_state, payload).await)
}

// 文件上传
#[post("/upload/file")]
pub async fn upload_file(
    app_state: web::Data<AppState>,
    payload: Multipart,
) -> ApiResponse<UploadFileResp> {
    ApiResponse::response(service::file::upload_file(&app_state, payload).await)
}

// 图片读取
#[get("/read/image/{filename}")]
pub async fn read_image(
    app_state: web::Data<AppState>,
    path: web::Path<(String,)>,
) -> actix_web::Result<HttpResponse> {
    service::file::read_image(&app_state, path.into_inner().0.as_str())
}

// 文件读取
#[get("/read/file/{filename}")]
pub async fn read_file(
    app_state: web::Data<AppState>,
    path: web::Path<(String,)>,
) -> actix_web::Result<HttpResponse> {
    service::file::read_file(&app_state, path.into_inner().0.as_str())
}

// 图片删除
#[post("/delete/file")]
pub async fn delete_file(
    app_state: web::Data<AppState>,
    req: web::Json<DeleteFileReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(service::file::delete_file(&app_state, req.into_inner()).await)
}
