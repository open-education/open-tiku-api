use crate::app::conf::AppState;
use crate::middleware::user::{ClientInfo, TeacherUserInfo, UserInfo};
use crate::service::user;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

// 用户相关接口

#[derive(Deserialize)]
pub struct ExchangeTokenReq {
    #[serde(rename(deserialize = "tempToken"))]
    pub temp_token: String,
}

// 换取用户登录 token
#[post("exchange")]
pub async fn exchange(
    app_state: web::Data<AppState>,
    req: web::Json<ExchangeTokenReq>,
) -> ApiResponse<String> {
    ApiResponse::response(user::exchange(&app_state, req.into_inner()).await)
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
    app_state: web::Data<AppState>,
    req: web::Json<UserLoginReq>,
    client_info: ClientInfo,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::login(&app_state, req.into_inner(), client_info).await)
}

// 通过 token 获取用户信息
#[get("info/{token}")]
pub async fn info(
    app_state: web::Data<AppState>,
    path: web::Path<(String,)>,
) -> ApiResponse<UserInfo> {
    ApiResponse::response(user::info(&app_state, path.into_inner().0.as_str()).await)
}

// 退出登录
#[get("logout")]
pub async fn logout(app_state: web::Data<AppState>, user_info: UserInfo) -> ApiResponse<bool> {
    ApiResponse::response(user::logout(&app_state, user_info).await)
}

// 第三方登录用户列表
#[derive(Deserialize)]
pub struct UserListReq {
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct UserIdentityInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub provider: i16,
    #[serde(rename(serialize = "providerDesc"))]
    pub provider_desc: String,
    #[serde(rename(serialize = "providerUsername"))]
    pub provider_username: String,
    #[serde(rename(serialize = "providerEmail"))]
    pub provider_email: String,
    #[serde(rename(serialize = "lastLoginTime"))]
    pub last_login_time: String,
    #[serde(rename(serialize = "loginCount"))]
    pub login_count: i64,
    pub role: i16,
    #[serde(rename(serialize = "roleDesc"))]
    pub role_desc: String,
    pub status: i16,
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub remark: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct UserListResp {
    pub list: Vec<UserIdentityInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("account/list")]
pub async fn account_list(
    app_state: web::Data<AppState>,
    req: web::Json<UserListReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<UserListResp> {
    ApiResponse::response(user::account_list(&app_state, req.into_inner()).await)
}

// Session 列表
#[derive(Deserialize)]
pub struct UserSessionListReq {
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct UserSessionInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    #[serde(rename(serialize = "sourceDesc"))]
    pub source_desc: String,
    pub username: String,
    #[serde(rename(serialize = "providerDesc"))]
    pub provider_desc: String,
    #[serde(rename(serialize = "expiredAt"))]
    pub expired_at: String,
    #[serde(rename(serialize = "renewCnt"))]
    pub renew_cnt: i16,
    #[serde(rename(serialize = "clientIp"))]
    pub client_ip: String,
    #[serde(rename(serialize = "userAgent"))]
    pub user_agent: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct UserSessionListResp {
    pub list: Vec<UserSessionInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("session/list")]
pub async fn session_list(
    app_state: web::Data<AppState>,
    req: web::Json<UserSessionListReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<UserSessionListResp> {
    ApiResponse::response(user::session_list(&app_state, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct UserEditReq {
    pub id: i64,
    pub status: i16,
    pub remark: String,
}

#[post("account/edit")]
pub async fn edit(
    app_state: web::Data<AppState>,
    req: web::Json<UserEditReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(user::edit(&app_state, req.into_inner()).await)
}
