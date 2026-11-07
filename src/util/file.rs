use crate::constant::meta;
use crate::util::error::AppError;
use actix_web::HttpResponse;
use std::fs;
use std::io::ErrorKind;
use tracing::warn;

pub fn read_file(
    meta_path: &str,
    is_image: bool,
    filename: &str,
) -> actix_web::Result<HttpResponse> {
    // 安全检查
    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return Ok(HttpResponse::BadRequest().body("无效的文件名"));
    }

    let file_path = if is_image {
        format!("{meta_path}/{}/{filename}", meta::IMAGE_NAME)
    } else {
        format!("{meta_path}/{}/{filename}", meta::FILE_NAME)
    };

    match fs::read(&file_path) {
        Ok(image_data) => {
            let content_type = get_content_type(filename);
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .body(image_data))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Ok(HttpResponse::NotFound().body("文件不存在"))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("读取文件失败")),
    }
}

fn get_content_type(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

pub async fn delete_file(
    meta_path: &str,
    is_image: bool,
    filename: &str,
) -> Result<bool, AppError> {
    // 安全检查
    if filename.is_empty()
        || filename.contains("..")
        || filename.contains('/')
        || filename.contains('\\')
    {
        return Err(AppError::param_error("无效的文件名"));
    }

    let file_path = if is_image {
        format!("{meta_path}/{}/{filename}", meta::IMAGE_NAME)
    } else {
        format!("{meta_path}/{}/{filename}", meta::FILE_NAME)
    };
    tokio::fs::remove_file(file_path).await.map_err(|e| {
        warn!("delete file error: {}", e);
        AppError::internal_error("删除文件失败")
    })?;

    Ok(true)
}
