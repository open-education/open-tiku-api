use crate::api::req::task::{TaskAddReq, TaskListReq};
use crate::api::resp::task::TaskListResp;
use crate::app::conf::AppState;
use crate::middleware::user::UserInfo;
use crate::service::task;
use crate::util::response::ApiResponse;
use actix_web::{post, web};

// 任务管理

// 创建任务
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<TaskAddReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(task::add(&app_state, req.into_inner(), user_info).await)
}

// 任务列表
#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<TaskListReq>,
) -> ApiResponse<TaskListResp> {
    ApiResponse::response(task::list(&app_state, req.into_inner()).await)
}
