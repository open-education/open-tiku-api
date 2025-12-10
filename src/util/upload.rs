use crate::api::file::UploadImageReq;
use crate::constant::meta;
use crate::util::string;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct UploadImageResp {
    pub original_name: String,
    pub size: usize,
    pub name: String,
    pub url: String,
}

/// 获取文件名
fn get_filename(field: &actix_multipart::Field) -> Result<String, Error> {
    let cd = field.content_disposition();
    match cd.get_filename() {
        Some(filename) => Ok(filename.to_string()),
        None => Err(Error::new(ErrorKind::Other, "无法获取文件名")),
    }
}

/// 验证文件类型
fn validate_file_type(filename: &str) -> Result<(), Error> {
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if !meta::ALLOW_IMAGE_EXTENSION.contains(&ext_lower.as_str()) {
            return Err(Error::new(
                ErrorKind::Other,
                format!("不支持的文件类型: {}", ext),
            ));
        }
    }
    Ok(())
}

/// 生成安全的文件名
fn generate_safe_filename(original_name: &str) -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%f");
    let safe_name = sanitize_filename::sanitize(original_name);
    (&format!("{:x}", md5::compute(format!("{}_{}", timestamp, safe_name)))[..meta::IMAGE_NAME_LEN])
        .to_string()
}

/// 保存文件
async fn save_file(field: &mut actix_multipart::Field, file_path: &str) -> Result<usize, Error> {
    let mut file = tokio::fs::File::create(file_path).await?;
    let mut file_size = 0;

    while let Some(chunk) = field.next().await {
        match chunk {
            Ok(chunk) => {
                if file_size + chunk.len() > meta::MAX_IMAGE_SIZE {
                    // 删除已创建的文件
                    let _ = tokio::fs::remove_file(file_path).await;
                    return Err(Error::new(
                        ErrorKind::FileTooLarge,
                        format!("文件大小超过限制: {}MB", meta::MAX_IMAGE_SIZE / 1024 / 1024),
                    ));
                }

                file_size += chunk.len();
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("save file error: {}", e),
                ));
            }
        }
    }

    Ok(file_size)
}

pub fn get_read_image_url(req_upload_file: &UploadImageReq, safe_name: &str) -> String {
    format!(
        "/api/file/read/{}/{}/{}",
        req_upload_file.textbook_key, req_upload_file.catalog_key, safe_name
    )
}

/// 处理文件上传
pub async fn upload_small_image(
    meta_path: &str,
    mut payload: Multipart,
    req_upload_file: UploadImageReq,
) -> Result<Vec<UploadImageResp>, Error> {
    let upload_path = format!(
        "{}/{}/{}/{}",
        meta_path,
        string::underline_to_slash(&req_upload_file.textbook_key),
        string::underline_to_slash(&req_upload_file.catalog_key),
        meta::IMAGE_NAME
    );

    // 确保上传目录存在
    std::fs::create_dir_all(&upload_path)?;

    let mut uploaded_files = Vec::new();

    while let Some(field) = payload.next().await {
        match field {
            Ok(mut field) => {
                let original_filename = get_filename(&field)?;

                // 验证文件类型
                validate_file_type(&original_filename)?;

                // 生成安全的文件名
                let safe_filename = generate_safe_filename(&original_filename);
                let file_path = format!("{}/{}", upload_path, safe_filename);

                // 保存文件
                let file_size = save_file(&mut field, &file_path).await?;

                uploaded_files.push(UploadImageResp {
                    original_name: original_filename,
                    name: safe_filename.clone(),
                    size: file_size,
                    url: get_read_image_url(&req_upload_file, &safe_filename),
                });
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("upload file error: {}", e),
                ));
            }
        }
    }

    Ok(uploaded_files)
}
