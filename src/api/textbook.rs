use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::textbook;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TextbookResp {
    pub id: i32,
    #[serde(rename(serialize = "pathType"))]
    pub path_type: String,
    #[serde(rename(serialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32, // 默认为 0
    #[serde(rename(serialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    pub path: String,
    #[serde(rename(serialize = "tableName"))]
    pub table_name: Option<String>,
    pub children: Option<Vec<TextbookResp>>,
}

// 根据深度获取所有父级菜单列表
#[get("/list/{depth}/all")]
pub async fn list_all(
    app_state: web::Data<AppState>,
    path: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_all(&app_state, path.into_inner().0).await)
}

// 获取指定深度的菜单标识获取子菜单列表
#[get("/list/{parent_id}/level")]
pub async fn list_level(
    app_state: web::Data<AppState>,
    parent_id: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_level(&app_state, parent_id.into_inner().0).await)
}

// 获取指定深度的所有子菜单列表-包括题型列表, 所以这个接口只是获取教材目录时有效
#[get("/list/{parent_id}/children")]
pub async fn list_children(
    app_state: web::Data<AppState>,
    parent_id: web::Path<(u32,)>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(textbook::list_children(&app_state, parent_id.into_inner().0).await)
}

// 新增时需要的字段（剔除 id 和 created_at）
#[derive(Deserialize)]
pub struct CreateTextbookReq {
    pub id: Option<i32>,
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    #[serde(rename(deserialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "pathType"))]
    pub path_type: Option<String>,
    pub path: String,
}

// 新增菜单
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<CreateTextbookReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<i32> {
    ApiResponse::response(textbook::add(&app_state, req.into_inner()).await)
}

// 删除菜单
#[get("/delete/{id}")]
pub async fn delete(
    app_state: web::Data<AppState>,
    path: web::Path<(i32,)>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(textbook::delete(&app_state, path.into_inner().0).await)
}
