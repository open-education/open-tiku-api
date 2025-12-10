use crate::constant::meta;
use crate::util::string;
use actix_web::HttpResponse;
use log::error;
use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;

// key: pep_math_senior_1
pub fn read_small_file<P: AsRef<Path>>(path: P, check_file_exist: bool) -> Result<String, Error> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        error!("path doesn't exist: {}", path_ref.display());
        return if check_file_exist {
            error!("The file: {} does not exist", path_ref.display());
            Err(Error::new(ErrorKind::NotFound, "File not found"))
        } else {
            Ok("".to_string())
        };
    }

    // 检查文件大小（可选：防止读取过大文件）
    let metadata = fs::metadata(path_ref)?;

    // 文件大小限制（例如 1MB）
    const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;
    if metadata.len() > MAX_FILE_SIZE {
        error!(
            "The file: {} is too large, currently only 2M can be read",
            path_ref.display()
        );
        return Err(Error::new(ErrorKind::FileTooLarge, "The file is too large"));
    }

    // 读取文件
    let mut file = File::open(path_ref)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn write_small_file(path: &str, content: &str) -> Result<bool, Error> {
    let path_obj = Path::new(path);
    let final_path = if path_obj.extension().is_none() {
        Path::new(&format!("{}.md", path)).to_path_buf()
    } else {
        path_obj.to_path_buf()
    };

    // 创建父目录
    if let Some(parent) = final_path.parent() {
        match fs::create_dir_all(parent) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create directory: {}", e);
                Err(Error::new(
                    ErrorKind::Interrupted,
                    format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e.to_string()
                    ),
                ))?
            }
        }
    }

    // 写入文件
    fs::write(&final_path, content)?;
    Ok(true)
}

pub struct LocalImageInfo {
    pub textbook_key: String,
    pub catalog_key: String,
    pub id: Option<String>,
    pub filename: String,
}

pub fn read_small_image(
    meta_path: &str,
    local_image_read: LocalImageInfo,
) -> actix_web::Result<HttpResponse> {
    let filename = local_image_read.filename;
    let image_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&local_image_read.textbook_key),
        string::underline_to_slash(&local_image_read.catalog_key),
        meta::IMAGE_NAME,
        filename
    );

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

pub async fn delete_image(
    meta_path: &str,
    local_image_info: LocalImageInfo,
) -> Result<bool, Error> {
    let image_path = format!(
        "{}/{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&local_image_info.textbook_key),
        string::underline_to_slash(&local_image_info.catalog_key),
        meta::IMAGE_NAME,
        local_image_info.filename
    );
    let _ = tokio::fs::remove_file(image_path).await?;
    Ok(true)
}
