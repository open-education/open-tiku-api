use crate::service::question_cate;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateQuestionCateReq {
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

#[derive(Serialize)]
pub struct QuestionCateResp {
    pub id: i32,
    #[serde(rename(serialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32,
}

// 添加题型
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateQuestionCateReq>,
) -> ApiResponse<QuestionCateResp> {
    ApiResponse::response(question_cate::add(app_conf, req.into_inner()).await)
}

// 题型列表
#[get("/list/{id}")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<Vec<QuestionCateResp>> {
    info!("req: {:?}", path);
    ApiResponse::response(question_cate::list(app_conf, path.into_inner().0).await)
}

// 删除题型
#[get("/remove/{id}")]
pub async fn remove(app_conf: web::Data<AppConfig>, path: web::Path<(i32,)>) -> ApiResponse<bool> {
    info!("req: {:?}", path);
    ApiResponse::response(question_cate::remove(app_conf, path.into_inner().0).await)
}
