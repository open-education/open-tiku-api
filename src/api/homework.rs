use crate::api::class_student::ClassStudentResp;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::homework;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 布置作业相关

// 获取试卷作业批次号
#[get("{paper_id}/batchNo")]
pub async fn batch_no(app_state: web::Data<AppState>, path: web::Path<(i64,)>) -> ApiResponse<i32> {
    ApiResponse::response(homework::batch_no(&app_state, path.into_inner().0).await)
}

// 布置作业添加
#[derive(Deserialize)]
pub struct HomeworkAddReq {
    #[serde(rename(deserialize = "batchNo"))]
    pub batch_no: i32,
    #[serde(rename(deserialize = "paperId"))]
    pub paper_id: i64,
    pub title: String,
    pub remark: Option<String>,
    #[serde(rename(deserialize = "classMap"))]
    pub class_map: HashMap<i64, Vec<i64>>,
}

#[post("add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<HomeworkAddReq>,
    teacher_user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(homework::add(&app_state, req.into_inner(), teacher_user_info).await)
}

// 查看作业布置详情列表
#[derive(Deserialize)]
pub struct HomeworkListReq {
    #[serde(rename(deserialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(deserialize = "batchNo"))]
    pub batch_no: Option<i32>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct HomeworkInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "batchNo"))]
    pub batch_no: i32,
    #[serde(rename(serialize = "homeworkId"))]
    pub homework_id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64,
    pub title: String,
    pub remark: String,
    pub students: Vec<ClassStudentResp>,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
}

#[derive(Serialize)]
pub struct HomeworkListResp {
    pub list: Vec<HomeworkInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<HomeworkListReq>,
    teacher_user_info: TeacherUserInfo,
) -> ApiResponse<HomeworkListResp> {
    ApiResponse::response(homework::list(&app_state, req.into_inner(), teacher_user_info).await)
}
