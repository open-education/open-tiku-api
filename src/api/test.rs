use crate::api::req::test::ListReq;
use crate::api::resp::test::ListResp;
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
