use crate::service::textbook;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
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

// 根据深度获取所有父级菜单列表
#[get("/list/{depth}/all")]
pub async fn list_all(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_all(app_conf, path.into_inner().0).await)
}

// 获取指定深度的所有子菜单列表
#[get("/list/{parent_id}/part")]
pub async fn list_part(
    app_conf: web::Data<AppConfig>,
    parent_id: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_part(app_conf, parent_id.into_inner().0).await)
}

// 获取指定深度的所有子菜单列表-包括题型列表, 所以这个接口只是获取教材目录时有效, 否则跟 /list/{parent_id}/part 一致
#[get("/list/{parent_id}/children")]
pub async fn list_children(
    app_conf: web::Data<AppConfig>,
    parent_id: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_children(app_conf, parent_id.into_inner().0).await)
}

// 新增时需要的字段（剔除 id 和 created_at）
#[derive(Deserialize)]
pub struct CreateTextbookReq {
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    #[serde(rename(deserialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

// 新增菜单
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateTextbookReq>,
) -> ApiResponse<TextbookResp> {
    ApiResponse::response(textbook::add(app_conf, req.into_inner()).await)
}

// 修改时需要的字段（通常包含 id，其他字段可选或必选）
#[derive(Deserialize)]
pub struct UpdateTextbookReq {
    pub id: i32,
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}

// 编辑菜单
#[post("/edit")]
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: web::Json<UpdateTextbookReq>,
) -> ApiResponse<TextbookResp> {
    ApiResponse::response(textbook::edit(app_conf, req.into_inner()).await)
}

// 获取菜单详情
#[get("/info/{id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<TextbookResp> {
    ApiResponse::response(textbook::info(app_conf, path.into_inner().0).await)
}

// 删除菜单
#[get("/delete/{id}")]
pub async fn delete(app_conf: web::Data<AppConfig>, path: web::Path<(i32,)>) -> ApiResponse<bool> {
    ApiResponse::response(textbook::delete(app_conf, path.into_inner().0).await)
}
