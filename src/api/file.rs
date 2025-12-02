use crate::util::file;
use crate::util::response::ApiResponse;
use crate::util::upload::{RespUploadImage, upload_small_file};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;

/// 文件上传请求
#[derive(Deserialize)]
pub struct UploadImageReq {
    pub textbook_key: String,
    pub catalog_key: String,
}

#[post("/upload")]
pub async fn upload(
    payload: Multipart,
    req: web::Query<UploadImageReq>,
) -> ApiResponse<Vec<RespUploadImage>> {
    ApiResponse::response(upload_small_file(payload, req.into_inner()).await)
}

#[get("/read/{textbook_key}/{catalog_key}/{filename}")]
pub async fn read(path: web::Path<(String, String, String)>) -> actix_web::Result<HttpResponse> {
    let path_into = path.into_inner();
    let read_small_file = file::LocalImageRead {
        textbook_key: path_into.0,
        catalog_key: path_into.1,
        filename: path_into.2,
    };
    file::read_small_image(read_small_file)
}

#[derive(Deserialize)]
pub struct DeleteImageReq {
    pub file_path: String,
}

#[post("/delete")]
pub async fn delete(req_delete_file: web::Json<DeleteImageReq>) -> ApiResponse<bool> {
    ApiResponse::response(file::delete_file(req_delete_file.into_inner()).await)
}
