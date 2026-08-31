use crate::api::req::class_student::{ClassStudentEditReq, ClassStudentListReq, ClassStudentReq};
use crate::api::resp::class_student::ClassStudentResp;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::class_student;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use std::collections::HashMap;

// 添加学生账户
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<ClassStudentReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<u64> {
    ApiResponse::response(class_student::add(&app_state, req.into_inner(), user_info).await)
}

// 获取班级的学生账户-不分页直接展示全部
#[post("list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<ClassStudentListReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<HashMap<i64, Vec<ClassStudentResp>>> {
    ApiResponse::response(class_student::list(&app_state, req.into_inner(), user_info).await)
}

// 编辑学生账户信息
#[post("/edit")]
pub async fn edit(
    app_state: web::Data<AppState>,
    req: web::Json<ClassStudentEditReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(class_student::edit(&app_state, req.into_inner(), user_info).await)
}
