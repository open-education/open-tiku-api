use crate::AppConfig;
use crate::service::callback;
use crate::util::response::ApiResponse;
use actix_web::{HttpResponse, Result, get, web};
use serde::Deserialize;

/// 第三方登录回调相关

// 用户登录时临时获取一个 state 值
#[get("{provider}/login/url")]
pub async fn login_url(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i16,)>,
) -> ApiResponse<String> {
    ApiResponse::response(callback::login_url(app_conf, path.into_inner().0).await)
}

// 回调参数
#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

// GitHub 登录回调
#[get("/github")]
pub async fn github(
    app_conf: web::Data<AppConfig>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse> {
    callback::github(app_conf, query.into_inner()).await
}

// QQ 登录回调
#[get("/qq")]
pub async fn qq(
    app_conf: web::Data<AppConfig>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse> {
    callback::qq(app_conf, query.into_inner()).await
}
