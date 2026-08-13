use crate::AppConfig;
use crate::middleware::user::UserInfo;
use crate::service::edit;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::Deserialize;

/// 编辑状态等

#[derive(Deserialize)]
pub struct CommonEditStatusReq {
    pub id: i64,
    pub status: i16,
    #[serde(rename(deserialize = "rejectReason"))]
    pub reject_reason: Option<String>,
}

// 更新题目状态
#[post("/question/status")]
pub async fn question_status(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CommonEditStatusReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::question_status(app_conf, req.into_inner(), user_info).await)
}

// 更新试卷状态
#[post("/paper/status")]
pub async fn paper_status(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CommonEditStatusReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::paper_status(app_conf, req.into_inner(), user_info).await)
}
