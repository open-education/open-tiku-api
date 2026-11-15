use crate::api::req::custom::CustomReq;
use crate::api::resp::custom::CustomResp;
use crate::app::conf::WebAppState;
use crate::middleware::user::UserInfo;
use crate::service::custom;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 个性化定制

// 保存个性化
#[post("/add")]
pub async fn add(
    app_state: web::Data<WebAppState>,
    req: web::Json<CustomReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(custom::add(&app_state, req.into_inner(), user_info).await)
}

#[get("/list")]
pub async fn list(
    app_state: web::Data<WebAppState>,
    user_info: Option<UserInfo>,
) -> ApiResponse<Vec<CustomResp>> {
    ApiResponse::response(custom::list(&app_state, user_info).await)
}
