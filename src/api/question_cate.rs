use crate::service::question_cate;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateQuestionCateReq {
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
pub struct QuestionCateResp {
    pub id: i32,
    #[serde(rename(serialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32,
}

#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateQuestionCateReq>,
) -> ApiResponse<QuestionCateResp> {
    info!("req: {:?}", req);
    ApiResponse::response(question_cate::add(app_conf, req.into_inner()).await)
}

#[get("/list/{id}")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<Vec<QuestionCateResp>> {
    info!("req: {:?}", path);
    ApiResponse::response(question_cate::list(app_conf, path.into_inner().0).await)
}

#[get("/remove/{id}")]
pub async fn remove(app_conf: web::Data<AppConfig>, path: web::Path<(i32,)>) -> ApiResponse<bool> {
    info!("req: {:?}", path);
    ApiResponse::response(question_cate::remove(app_conf, path.into_inner().0).await)
}
