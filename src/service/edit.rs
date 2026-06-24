use crate::AppConfig;
use crate::api::edit::EditStatusReq;
use crate::model::question::Question;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 更新状态
pub async fn status(app_conf: web::Data<AppConfig>, req: EditStatusReq) -> Result<bool, Error> {
    let approve_id = 1;

    let row = Question::update_status_by_id(
        &app_conf.get_ref().db,
        req.id,
        req.status,
        approve_id,
        req.reject_reason,
    )
    .await
    .map_err(|e| {
        error!("Error while updating Status: {:?}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    Ok(row > 0)
}
