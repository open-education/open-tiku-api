use crate::AppConfig;
use crate::api::file::{DeleteImageReq, UploadImageReq};
use crate::service::edit;
use crate::util::{file, upload};
use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use std::io::Error;

// 上传
pub async fn upload_small_image(
    app_conf: web::Data<AppConfig>,
    payload: Multipart,
    req: UploadImageReq,
) -> Result<Vec<upload::UploadImageResp>, Error> {
    let resp = upload::upload_small_image(&app_conf.get_ref().meta_path, payload).await?;

    // 添加题目对应的图片
    if let Some(filename) = resp.get(0)
        && let Some(id) = req.id
    {
        edit::add_image(&app_conf.get_ref().db, id, filename.name.as_str()).await?;
    }

    Ok(resp)
}

// 读取
pub fn read_small_image(
    app_conf: web::Data<AppConfig>,
    filename: &str,
) -> actix_web::Result<HttpResponse> {
    file::read_small_image(app_conf.meta_path.as_str(), filename)
}

// 删除
pub async fn delete_image(
    app_conf: web::Data<AppConfig>,
    req: DeleteImageReq,
) -> Result<bool, Error> {
    file::delete_image(&app_conf.get_ref().meta_path, req.filename.as_str()).await?;

    // 删除题目对应的图片
    if let Some(id) = req.id {
        edit::remove_image(&app_conf.get_ref().db, id, req.filename.as_str()).await
    } else {
        Ok(true)
    }
}
