// 用户相关接口

use crate::AppConfig;
use crate::middleware::user::UserInfo;
use crate::service::user;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ExchangeTokenReq {
    #[serde(rename(deserialize = "tempToken"))]
    pub temp_token: String,
}

// 换取用户登录 token
#[post("exchange")]
pub async fn exchange(
    app_conf: web::Data<AppConfig>,
    req: web::Json<ExchangeTokenReq>,
) -> ApiResponse<String> {
    ApiResponse::response(user::exchange(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct UserLoginReq {
    pub token: String,
}

// 用户登录
#[post("login")]
pub async fn login(
    app_conf: web::Data<AppConfig>,
    req: web::Json<UserLoginReq>,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::login(app_conf, req.into_inner()).await)
}

// 通过 token 获取用户信息
#[get("info/{token}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::info(app_conf, path.into_inner().0.as_str()).await)
}

// 退出登录
#[get("logout")]
pub async fn logout(app_conf: web::Data<AppConfig>, user_info: UserInfo) -> ApiResponse<bool> {
    ApiResponse::response(user::logout(app_conf, user_info).await)
}
