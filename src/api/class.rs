use crate::api::req::class::ClassInfoReq;
use crate::api::resp::class::{ClassListReq, ClassListResp};
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::class;
use crate::util::response::ApiResponse;
use actix_web::{post, web};

// 班级管理

// 班级添加
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<ClassInfoReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(class::add(&app_state, req.into_inner(), user_info).await)
}

// 班级列表
#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<ClassListReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<ClassListResp> {
    ApiResponse::response(class::list(&app_state, req.into_inner(), user_info).await)
}
