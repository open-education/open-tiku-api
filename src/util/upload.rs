use crate::constant::meta;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use serde::{Serialize};
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Serialize)]
pub struct UploadFileResp {
    #[serde(rename(serialize = "originalName"))]
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
fn validate_file_type(filename: &str, is_image: bool) -> Result<(), Error> {
    // 1. 获取小写后缀，若无后缀则直接报错
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "文件缺少扩展名"))?;

    // 2. 选择匹配列表
    let (list, type_desc) = if is_image {
        (&meta::ALLOW_IMAGE_EXTENSION[..], "图片")
    } else {
        (&meta::ALLOW_FILE_EXTENSION[..], "文件")
    };

    // 3. 校验
    if !list.contains(&ext.as_str()) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("不支持的{}类型: .{}", type_desc, ext),
        ));
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

// 读取图片的路径要加上 nginx 代理指定的前缀, 此时默认为 api
pub fn get_read_file_url(safe_name: &str, is_image: bool) -> String {
    if is_image {
        format!("/{}/file/read/image/{}", meta::IMAGE_READ_PREFIX, safe_name)
    } else {
        format!("/{}/file/read/file/{}", meta::IMAGE_READ_PREFIX, safe_name)
    }
}

// 实际上传文件至本地
pub async fn upload_file(
    meta_path: &str,
    mut payload: Multipart,
    is_image: &bool,
) -> Result<UploadFileResp, Error> {
    let upload_path = format!(
        "{}/{}",
        meta_path,
        if *is_image {
            meta::IMAGE_NAME
        } else {
            meta::FILE_NAME
        }
    );
    std::fs::create_dir_all(&upload_path)?;

    // 获取第一个有效字段, 实际上只上传一个文件
    let mut field = payload
        .next()
        .await
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "No file uploaded"))?
        .map_err(|e| Error::new(ErrorKind::Other, format!("Upload error: {}", e)))?;

    let original_filename = get_filename(&field)?;

    // 验证与生成文件名
    validate_file_type(&original_filename, *is_image)?;
    let safe_filename = generate_safe_filename(&original_filename);
    let file_path = format!("{}/{}", upload_path, safe_filename);

    // 保存
    let file_size = save_file(&mut field, &file_path).await?;

    Ok(UploadFileResp {
        original_name: original_filename,
        name: safe_filename.clone(),
        size: file_size,
        url: get_read_file_url(&safe_filename, *is_image),
    })
}
