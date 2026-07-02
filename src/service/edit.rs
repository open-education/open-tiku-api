use crate::AppConfig;
use crate::api::edit::EditStatusReq;
use crate::middleware::user::UserInfo;
use crate::model::question::Question;
use crate::model::user_identity::RoleType;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 更新状态
pub async fn status(
    app_conf: web::Data<AppConfig>,
    req: EditStatusReq,
    user_info: UserInfo,
) -> Result<bool, Error> {
    // 只要教师有审核权限
    if user_info.role != RoleType::Teacher.as_i16() {
        return Err(Error::new(ErrorKind::Other, "你的账户角色没有审核权限"));
    }
    let approve_id = user_info.user_id;

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
