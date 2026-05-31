use crate::AppConfig;
use crate::service;
use crate::util::response::ApiResponse;
use crate::util::upload::UploadFileResp;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;

/// 上传请求

// 图片上传
#[post("/upload/image")]
pub async fn upload_image(
    app_conf: web::Data<AppConfig>,
    payload: Multipart,
) -> ApiResponse<UploadFileResp> {
    ApiResponse::response(service::file::upload_image(app_conf, payload).await)
}

// 文件上传
#[post("/upload/file")]
pub async fn upload_file(
    app_conf: web::Data<AppConfig>,
    payload: Multipart,
) -> ApiResponse<UploadFileResp> {
    ApiResponse::response(service::file::upload_file(app_conf, payload).await)
}

// 图片读取
#[get("/read/image/{filename}")]
pub async fn read_image(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> actix_web::Result<HttpResponse> {
    service::file::read_image(app_conf, path.into_inner().0.as_str())
}

// 文件读取
#[get("/read/file/{filename}")]
pub async fn read_file(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> actix_web::Result<HttpResponse> {
    service::file::read_file(app_conf, path.into_inner().0.as_str())
}

#[derive(Deserialize)]
pub struct DeleteImageReq {
    pub filename: String,
}

// 图片删除
#[post("/delete/image")]
pub async fn delete_image(
    app_conf: web::Data<AppConfig>,
    req: web::Json<DeleteImageReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(service::file::delete_image(app_conf, req.into_inner()).await)
}
