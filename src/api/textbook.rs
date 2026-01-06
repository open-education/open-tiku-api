use crate::service::textbook;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TextbookResp {
    pub id: i32,
    #[serde(rename(serialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32, // 默认为 0
    #[serde(rename(serialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    pub children: Option<Vec<TextbookResp>>,
}

#[get("/list/{depth}/all")]
pub async fn list_all(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_all(app_conf, path.into_inner().0).await)
}

#[get("/list/{parent_id}/part")]
pub async fn list_part(
    app_conf: web::Data<AppConfig>,
    parent_id: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_part(app_conf, parent_id.into_inner().0).await)
}

// 新增时需要的字段（剔除 id 和 created_at）
#[derive(Deserialize, Debug)]
pub struct CreateTextbookReq {
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    pub key: String,
    #[serde(rename(deserialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateTextbookReq>,
) -> ApiResponse<TextbookResp> {
    info!("re: {:?}", req);
    ApiResponse::response(textbook::add(app_conf, req.into_inner()).await)
}

// 修改时需要的字段（通常包含 id，其他字段可选或必选）
#[derive(Deserialize, Debug)]
pub struct UpdateTextbookReq {
    pub id: i32,
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub key: String,
    pub label: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

#[post("/edit")]
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: web::Json<UpdateTextbookReq>,
) -> ApiResponse<TextbookResp> {
    info!("re: {:?}", req);
    ApiResponse::response(textbook::edit(app_conf, req.into_inner()).await)
}

#[get("/info/{id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<TextbookResp> {
    ApiResponse::response(textbook::info(app_conf, path.into_inner().0).await)
}

#[get("/delete/{id}")]
pub async fn delete(app_conf: web::Data<AppConfig>, path: web::Path<(i32,)>) -> ApiResponse<bool> {
    ApiResponse::response(textbook::delete(app_conf, path.into_inner().0).await)
}
