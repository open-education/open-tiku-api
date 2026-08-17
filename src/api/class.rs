use crate::app::config::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::class;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::{Deserialize, Serialize};
// 班级管理

// 班级添加
#[derive(Deserialize)]
pub struct ClassInfoReq {
    pub id: Option<i64>,
    pub year: String,
    pub grade: Option<String>,
    pub semester: Option<String>,
    pub label: String,
    pub email: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i16,
    pub remark: String,
}

// 班级添加
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<ClassInfoReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(class::add(app_state, req.into_inner(), user_info).await)
}

#[derive(Deserialize)]
pub struct ClassListReq {
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

// 班级信息返回
#[derive(Serialize)]
pub struct ClassInfoResp {
    pub id: i64,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub label: String,
    pub email: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i16,
    pub remark: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ClassListResp {
    pub list: Vec<ClassInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

// 班级列表
#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<ClassListReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<ClassListResp> {
    ApiResponse::response(class::list(app_state, req.into_inner(), user_info).await)
}
