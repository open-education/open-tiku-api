use crate::constant::meta;
use actix_web::HttpResponse;
use std::fs;
use std::io::{Error, ErrorKind};

pub fn read_small_image(meta_path: &str, filename: &str) -> actix_web::Result<HttpResponse> {
    let image_path = format!("{}/{}/{}", meta_path, meta::IMAGE_NAME, filename);

    // 安全检查
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Ok(HttpResponse::BadRequest().body("无效的文件名"));
    }

    match fs::read(&image_path) {
        Ok(image_data) => {
            let content_type = get_content_type(&filename);
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(image_data))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Ok(HttpResponse::NotFound().body("图片不存在"))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("读取图片失败")),
    }
}

fn get_content_type(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

pub async fn delete_image(meta_path: &str, filename: &str) -> Result<bool, Error> {
    let image_path = format!("{}/{}/{}", meta_path, meta::IMAGE_NAME, filename);
    let _ = tokio::fs::remove_file(image_path).await?;
    Ok(true)
}
