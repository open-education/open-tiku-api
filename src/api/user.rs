use crate::api::req::user::{
    ExchangeTokenReq, UserEditReq, UserListReq, UserLoginReq, UserSessionListReq,
};
use crate::api::resp::user::{UserListResp, UserSessionListResp};
use crate::app::conf::AppState;
use crate::middleware::user::{ClientInfo, TeacherUserInfo, UserInfo};
use crate::service::user;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 用户相关接口

// 换取用户登录 token
#[post("exchange")]
pub async fn exchange(
    app_state: web::Data<AppState>,
    req: web::Json<ExchangeTokenReq>,
) -> ApiResponse<String> {
    ApiResponse::response(user::exchange(&app_state, req.into_inner()).await)
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
#[post("account/list")]
pub async fn account_list(
    app_state: web::Data<AppState>,
    req: web::Json<UserListReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<UserListResp> {
    ApiResponse::response(user::account_list(&app_state, req.into_inner()).await)
}

// Session 列表
#[post("session/list")]
pub async fn session_list(
    app_state: web::Data<AppState>,
    req: web::Json<UserSessionListReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<UserSessionListResp> {
    ApiResponse::response(user::session_list(&app_state, req.into_inner()).await)
}

#[post("account/edit")]
pub async fn edit(
    app_state: web::Data<AppState>,
    req: web::Json<UserEditReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(user::edit(&app_state, req.into_inner()).await)
}
