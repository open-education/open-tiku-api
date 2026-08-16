use crate::app::config::AppState;
use crate::middleware::user::{ClientInfo, UserInfo};
use crate::service::user;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::Deserialize;

// 用户相关接口

#[derive(Deserialize)]
pub struct ExchangeTokenReq {
    #[serde(rename(deserialize = "tempToken"))]
    pub temp_token: String,
}

// 换取用户登录 token
#[post("exchange")]
pub async fn exchange(
    app_conf: web::Data<AppState>,
    req: web::Json<ExchangeTokenReq>,
) -> ApiResponse<String> {
    ApiResponse::response(user::exchange(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct UserLoginReq {
    // 登录来源 1 第三方用户 2 学生账户
    pub source: i16,

    // 临时 token 登录
    pub token: Option<String>,

    // 用户名密码登录
    pub account: Option<String>,
    pub password: Option<String>,
}

// 第三方用户登录
#[post("login")]
pub async fn login(
    app_conf: web::Data<AppState>,
    req: web::Json<UserLoginReq>,
    client_info: ClientInfo,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::login(app_conf, req.into_inner(), client_info).await)
}

// 通过 token 获取用户信息
#[get("info/{token}")]
pub async fn info(
    app_conf: web::Data<AppState>,
    path: web::Path<(String,)>,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::info(app_conf, path.into_inner().0.as_str()).await)
}

// 退出登录
#[get("logout")]
pub async fn logout(app_conf: web::Data<AppState>, user_info: UserInfo) -> ApiResponse<bool> {
    ApiResponse::response(user::logout(app_conf, user_info).await)
}
