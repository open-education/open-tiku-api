use crate::api::req::test::{LatestAttemptReq, ListReq, TestAnswerAddReq};
use crate::api::resp::test::{AttemptInfoResp, ListResp};
use crate::app::conf::AppState;
use crate::middleware::user::StudentUserInfo;
use crate::service::test;
use crate::util::response::ApiResponse;
use actix_web::{post, web};

// 学生练习今日今日任务
#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<ListReq>,
    user_info: StudentUserInfo,
) -> ApiResponse<ListResp> {
    ApiResponse::response(test::list(&app_state, req.into_inner(), user_info).await)
}

// 进行中的做题记录
#[post("/attempt/latest")]
pub async fn attempt_latest(
    app_state: web::Data<AppState>,
    req: web::Json<LatestAttemptReq>,
    user_info: StudentUserInfo,
) -> ApiResponse<AttemptInfoResp> {
    ApiResponse::response(test::attempt_latest(&app_state, req.into_inner(), user_info).await)
}

// 保存答案
#[post("/answer/add")]
pub async fn answer_add(
    app_state: web::Data<AppState>,
    req: web::Json<TestAnswerAddReq>,
    user_info: StudentUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(test::answer_add(&app_state, req.into_inner(), user_info).await)
}
