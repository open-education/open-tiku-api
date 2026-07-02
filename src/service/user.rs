use crate::AppConfig;
use crate::api::user::{ExchangeTokenReq, UserLoginReq};
use crate::constant::meta;
use crate::middleware::user::{UserInfo, get_user_identity, get_user_session};
use crate::model::user_identity::UserIdentity;
use crate::model::user_session::{TokenType, UserSession};
use actix_web::web;
use chrono::{Duration, Utc};
use log::error;
use std::io::{Error, ErrorKind};
use uuid::Uuid;

// 换取登录 token
pub async fn exchange(
    app_conf: web::Data<AppConfig>,
    req: ExchangeTokenReq,
) -> Result<String, Error> {
    let db = &app_conf.db;

    // session 信息
    let mut session =
        get_user_session(db, req.temp_token.as_str(), TokenType::Temp.as_i16()).await?;

    // 用户信息
    let _ = get_user_identity(db, session.user_id).await?;

    // 替换 session 为登录 token
    let login_token = Uuid::new_v4().to_string();
    session.token = login_token.clone();
    session.expired_at = Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE);
    session.token_type = TokenType::Login.as_i16();

    // 替换用户临时 session 为 登录 session
    let _ = UserSession::save(db, session).await.map_err(|err| {
        error!("Exchange save user session save err: {}", err);
        Error::new(ErrorKind::InvalidInput, "更新用户session信息错误")
    })?;

    Ok(login_token)
}

// 登录
pub async fn login(app_conf: web::Data<AppConfig>, req: UserLoginReq) -> Result<UserInfo, Error> {
    let db = &app_conf.db;

    // session 信息
    let mut session = get_user_session(db, req.token.as_str(), TokenType::Login.as_i16()).await?;

    // 用户信息
    let mut user = get_user_identity(db, session.user_id).await?;

    // 更新 session
    session.expired_at = Utc::now() + Duration::hours(meta::LOGIN_TOKEN_EXPIRED_HOUR);
    session.token_type = TokenType::Login.as_i16();
    session.renew_cnt = session.renew_cnt + 1;
    session.use_cnt = session.use_cnt + 1;

    // 更新用户 session
    let _ = UserSession::save(db, session).await.map_err(|err| {
        error!("Login save user session save err: {}", err);
        Error::new(ErrorKind::InvalidInput, "更新用户session信息错误")
    })?;

    // 更新用户统计信息
    user.last_login_time = Some(Utc::now());
    user.login_count = user.login_count + 1;
    let _ = UserIdentity::save(db, &user).await.map_err(|err| {
        error!("Login save user session save err: {}", err);
        Error::new(ErrorKind::InvalidInput, "更新用户信息错误")
    })?;

    Ok(UserInfo {
        user_id: user.user_id,
        username: user.provider_username,
        email: user.provider_email,
        role: user.role,
        status: user.status,
        token: None,
    })
}

// 获取用户信息
pub async fn info(app_conf: web::Data<AppConfig>, token: &str) -> Result<UserInfo, Error> {
    let db = &app_conf.db;

    // session 信息
    let session = get_user_session(db, token, TokenType::Login.as_i16()).await?;

    // 用户信息
    let user = get_user_identity(db, session.user_id).await?;

    Ok(UserInfo {
        user_id: user.user_id,
        username: user.provider_username,
        email: user.provider_email,
        role: user.role,
        status: user.status,
        token: None,
    })
}

// 退出登录
pub async fn logout(app_conf: web::Data<AppConfig>, user_info: UserInfo) -> Result<bool, Error> {
    let db = &app_conf.db;

    // session 信息
    let session = get_user_session(
        db,
        user_info.token.unwrap_or_default().as_str(),
        TokenType::Login.as_i16(),
    )
    .await
    .map_err(|err| {
        error!("Login save user session save err: {}", err);
        Error::new(ErrorKind::Other, err)
    })?;

    UserSession::delete_by_id(db, session.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!("Login delete user session save err: {}", err);
            Error::new(ErrorKind::Other, err)
        })?;

    Ok(true)
}
