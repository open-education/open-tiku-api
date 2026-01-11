use crate::service::file::{delete_image, read_small_image, upload_small_image};
use crate::util::response::ApiResponse;
use crate::util::upload::UploadImageResp;
use crate::AppConfig;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

/// 文件上传请求

#[derive(Deserialize)]
pub struct UploadImageReq {
    pub id: Option<i64>,
}

// 图片上传
#[post("/upload")]
pub async fn upload(
    app_conf: web::Data<AppConfig>,
    payload: Multipart,
    req: web::Query<UploadImageReq>,
) -> ApiResponse<Vec<UploadImageResp>> {
    ApiResponse::response(upload_small_image(app_conf, payload, req.into_inner()).await)
}

// 图片读取
#[get("/read/{filename}")]
pub async fn read(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> actix_web::Result<HttpResponse> {
    read_small_image(app_conf, path.into_inner().0.as_str())
}

#[derive(Deserialize)]
pub struct DeleteImageReq {
    pub id: Option<i64>,
    pub filename: String,
}

// 图片删除
#[post("/delete")]
pub async fn delete(
    app_conf: web::Data<AppConfig>,
    req: web::Json<DeleteImageReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(delete_image(app_conf, req.into_inner()).await)
}
