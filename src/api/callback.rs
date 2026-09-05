use crate::api::req::callback::CallbackQueryReq;
use crate::app::conf::AppState;
use crate::service::callback;
use crate::util::response::ApiResponse;
use actix_web::{HttpResponse, Result, get, web};

// 第三方登录回调相关

// 用户登录时临时获取一个 state 值
#[get("{provider}/login/url")]
pub async fn login_url(
    app_state: web::Data<AppState>,
    path: web::Path<(i16,)>,
) -> ApiResponse<String> {
    ApiResponse::response(callback::login_url(&app_state, path.into_inner().0).await)
}

// GitHub 登录回调
#[get("/github")]
pub async fn github(
    app_state: web::Data<AppState>,
    query: web::Query<CallbackQueryReq>,
) -> Result<HttpResponse> {
    callback::github(&app_state, query.into_inner()).await
}

// QQ 登录回调
#[get("/qq")]
pub async fn qq(
    app_state: web::Data<AppState>,
    query: web::Query<CallbackQueryReq>,
) -> Result<HttpResponse> {
    callback::qq(&app_state, query.into_inner()).await
}
