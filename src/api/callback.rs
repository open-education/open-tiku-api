use crate::AppConfig;
use crate::service::callback;
use actix_web::{HttpResponse, Result, get, web};
use serde::Deserialize;

/// 第三方登录回调相关

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
#[get("/github")]
pub async fn qq(
    app_conf: web::Data<AppConfig>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse> {
    callback::qq(app_conf, query.into_inner()).await
}
