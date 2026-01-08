use crate::service::textbook_dict;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateTextbookDictReq {
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(deserialize = "typeCode"))]
    pub type_code: String,
    #[serde(rename(deserialize = "itemValue"))]
    pub item_value: String,
    pub sort_order: i32,
}

#[derive(Serialize)]
pub struct TextbookDictResp {
    pub id: i32,
    #[serde(rename(serialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(serialize = "typeCode",))]
    pub type_code: String,
    #[serde(rename(serialize = "itemValue"))]
    pub item_value: String,
    pub sort_order: i32,
}

// 字典添加
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateTextbookDictReq>,
) -> ApiResponse<TextbookDictResp> {
    ApiResponse::response(textbook_dict::add(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct TextbookDictListReq {
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(deserialize = "typeCode"))]
    pub type_code: String,
}

// 字典查询
#[get("/{id}")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<TextbookDictListReq>,
) -> ApiResponse<Vec<TextbookDictResp>> {
    ApiResponse::response(textbook_dict::get_list(app_conf, req.into_inner()).await)
}

// 字典删除
#[get("/remove/{id}")]
pub async fn remove(app_conf: web::Data<AppConfig>, path: web::Path<(i32,)>) -> ApiResponse<bool> {
    ApiResponse::response(textbook_dict::delete(app_conf, path.into_inner().0).await)
}
