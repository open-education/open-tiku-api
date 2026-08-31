use crate::api::req::other_dict::CreateTextbookDictReq;
use crate::api::resp::other_dict::TextbookDictResp;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::textbook_dict;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

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
