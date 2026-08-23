use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::class_student;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ClassStudentReq {
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    // 是否增量导入
    pub incremental: bool,
    // 账户名称是英文逗号分割的字符串
    pub accounts: String,
}

// 添加学生账户
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<ClassStudentReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<u64> {
    ApiResponse::response(class_student::add(&app_state, req.into_inner(), user_info).await)
}

#[derive(Serialize)]
pub struct ClassStudentResp {
    pub id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub account: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub remark: String,
    #[serde(rename(serialize = "lastLoginTime"))]
    pub last_login_time: String,
    #[serde(rename(serialize = "loginCount"))]
    pub login_count: i64,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

// 获取班级的学生账户-不分页直接展示全部
#[derive(Deserialize)]
pub struct ClassStudentListReq {
    #[serde(rename(deserialize = "classIds"))]
    pub class_ids: Vec<i64>,
}

#[post("list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<ClassStudentListReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<HashMap<i64, Vec<ClassStudentResp>>> {
    ApiResponse::response(class_student::list(&app_state, req.into_inner(), user_info).await)
}

// 修改学生账户信息
#[derive(Deserialize)]
pub struct ClassStudentEditReq {
    pub id: i64,
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    pub account: String,
    pub status: i16,
    #[serde(rename(deserialize = "resetPwd"))]
    pub reset_pwd: bool,
    pub remark: String,
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
