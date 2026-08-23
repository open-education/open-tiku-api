use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::textbook_dict;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateTextbookDictReq {
    pub id: Option<i32>,
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(deserialize = "typeCode"))]
    pub type_code: String,
    #[serde(rename(deserialize = "itemValue"))]
    pub item_value: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "isSelect"))]
    pub is_select: bool,
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
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(serialize = "isSelect"))]
    pub is_select: bool,
}

// 字典添加
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<CreateTextbookDictReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<i32> {
    ApiResponse::response(textbook_dict::add(&app_state, req.into_inner()).await)
}

// 字典查询
#[get("/list/{textbook_id}/{type_code}")]
pub async fn list(
    app_state: web::Data<AppState>,
    path: web::Path<(i32, String)>,
) -> ApiResponse<Vec<TextbookDictResp>> {
    let path = path.into_inner();
    ApiResponse::response(textbook_dict::get_list(&app_state, path.0, path.1).await)
}

// 字典删除
#[get("/remove/{id}")]
pub async fn remove(
    app_state: web::Data<AppState>,
    path: web::Path<(i32,)>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(textbook_dict::delete(&app_state, path.into_inner().0).await)
}
