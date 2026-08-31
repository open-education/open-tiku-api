use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::homework;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

use crate::api::req::homework::{HomeworkAddReq, HomeworkListReq};
use crate::api::resp::homework::HomeworkListResp;

// 布置作业相关

// 获取试卷作业批次号
#[get("{paper_id}/batchNo")]
pub async fn batch_no(app_state: web::Data<AppState>, path: web::Path<(i64,)>) -> ApiResponse<i32> {
    ApiResponse::response(homework::batch_no(&app_state, path.into_inner().0).await)
}

// 布置作业添加
#[post("add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<HomeworkAddReq>,
    teacher_user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(homework::add(&app_state, req.into_inner(), teacher_user_info).await)
}

// 查看作业布置详情列表
#[post("list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<HomeworkListReq>,
    teacher_user_info: TeacherUserInfo,
) -> ApiResponse<HomeworkListResp> {
    ApiResponse::response(homework::list(&app_state, req.into_inner(), teacher_user_info).await)
}
