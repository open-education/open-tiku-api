use crate::AppConfig;
use crate::service::task;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::{Deserialize, Serialize};

/// 任务管理

#[derive(Deserialize)]
pub struct TaskAddReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(deserialize = "taskType"))]
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
}

// 创建任务
#[post("/add")]
pub async fn add(app_conf: web::Data<AppConfig>, req: web::Json<TaskAddReq>) -> ApiResponse<i64> {
    ApiResponse::response(task::add(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct TaskListReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(deserialize = "taskType"))]
    pub task_type: i16,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct TaskInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(serialize = "taskType"))]
    pub task_type: i16,
    pub name: String,
    pub author: String,
    pub email: String,
    pub status: i16,
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub result: Option<String>,
    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct TaskListResp {
    pub list: Vec<TaskInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

// 任务列表
#[post("/list")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<TaskListReq>,
) -> ApiResponse<TaskListResp> {
    ApiResponse::response(task::list(app_conf, req.into_inner()).await)
}
