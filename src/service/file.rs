use crate::api::file::DeleteFileReq;
use crate::app::config::AppState;
use crate::util::error::AppError;
use crate::util::{file, upload};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use log::error;

// 上传图片
pub async fn upload_image(
    app_state: web::Data<AppState>,
    payload: Multipart,
) -> Result<upload::UploadFileResp, AppError> {
    let resp = upload::upload_file(&app_state.config.meta.path, payload, &true).await?;

    Ok(resp)
}

// 上传文件
pub async fn upload_file(
    app_state: web::Data<AppState>,
    payload: Multipart,
) -> Result<upload::UploadFileResp, AppError> {
    let resp = upload::upload_file(&app_state.config.meta.path, payload, &false).await?;

    Ok(resp)
}

// 读取图片
pub fn read_image(
    app_state: web::Data<AppState>,
    filename: &str,
) -> actix_web::Result<HttpResponse> {
    file::read_file(&app_state.config.meta.path, true, filename)
}

// 读取文件
pub fn read_file(
    app_state: web::Data<AppState>,
    filename: &str,
) -> actix_web::Result<HttpResponse> {
    file::read_file(&app_state.config.meta.path, false, filename)
}

// 删除
pub async fn delete_file(
    app_state: web::Data<AppState>,
    req: DeleteFileReq,
) -> Result<bool, AppError> {
    file::delete_file(
        &app_state.config.meta.path,
        req.is_image,
        req.filename.as_str(),
    )
    .await
    .map_err(|err| {
        error!("delete file : {} err: {}", req.filename, err);
        AppError::internal_error("文件删除失败")
    })?;

    Ok(true)
}
