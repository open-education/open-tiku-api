use crate::AppConfig;
use crate::service::edit;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::Deserialize;

/// 编辑

#[derive(Deserialize)]
pub struct EditStatusReq {
    pub id: i64,
    pub status: i16,
    #[serde(rename(deserialize = "rejectReason"))]
    pub reject_reason: Option<String>,
}

// 更新状态
#[post("/status")]
pub async fn status(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditStatusReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::status(app_conf, req.into_inner()).await)
}
