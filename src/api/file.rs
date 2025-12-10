use crate::util::file;
use crate::util::response::ApiResponse;
use crate::util::upload::UploadImageResp;
use crate::{AppConfig, service};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;

/// 文件上传请求
#[derive(Deserialize)]
pub struct UploadImageReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: Option<String>,
}

#[post("/upload")]
pub async fn upload(
    app_conf: web::Data<AppConfig>,
    payload: Multipart,
    req: web::Query<UploadImageReq>,
) -> ApiResponse<Vec<UploadImageResp>> {
    ApiResponse::response(
        service::file::upload_small_image(
            app_conf.meta_path.to_str().unwrap(),
            payload,
            req.into_inner(),
        )
        .await,
    )
}

#[get("/read/{textbook_key}/{catalog_key}/{filename}")]
pub async fn read(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String, String, String)>,
) -> actix_web::Result<HttpResponse> {
    let path_into = path.into_inner();
    let read_small_file = file::LocalImageInfo {
        textbook_key: path_into.0,
        catalog_key: path_into.1,
        id: None,
        filename: path_into.2,
    };
    file::read_small_image(app_conf.meta_path.to_str().unwrap(), read_small_file)
}

#[derive(Deserialize)]
pub struct DeleteImageReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: Option<String>,
    pub filename: String,
}

#[post("/delete")]
pub async fn delete(
    app_conf: web::Data<AppConfig>,
    path: web::Json<DeleteImageReq>,
) -> ApiResponse<bool> {
    let path_into = path.into_inner();
    let delete_small_file = file::LocalImageInfo {
        textbook_key: path_into.textbook_key,
        catalog_key: path_into.catalog_key,
        id: path_into.id,
        filename: path_into.filename,
    };
    ApiResponse::response(
        service::file::delete_image(app_conf.meta_path.to_str().unwrap(), delete_small_file).await,
    )
}
