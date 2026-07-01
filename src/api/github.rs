use crate::AppConfig;
use crate::service::github;
use actix_web::{HttpResponse, Result, get, web};
use serde::Deserialize;

/// 第三方登录回调相关

#[derive(Deserialize)]
pub struct GitHubCallbackQuery {
    pub code: Option<String>,
}

// GitHub 登录回调
#[get("/github")]
pub async fn callback(
    app_conf: web::Data<AppConfig>,
    query: web::Query<GitHubCallbackQuery>,
) -> Result<HttpResponse> {
    github::callback(app_conf, query.into_inner()).await
}
