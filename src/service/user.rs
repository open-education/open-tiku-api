use crate::AppConfig;
use crate::api::user::{ExchangeTokenReq, UserInfoResp, UserLoginReq};
use crate::constant::meta;
use crate::model::user_identity::{StatusType, UserIdentity};
use crate::model::user_session::{SourceType, TokenType, UserSession};
use actix_web::web;
use chrono::{Duration, Utc};
use log::error;
use sqlx::PgPool;
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

// 根据 token 获取用户 session 信息
async fn get_user_session(db: &PgPool, token: &str, token_type: i16) -> Result<UserSession, Error> {
    let session = UserSession::find_one_by_token(db, token, token_type)
        .await
        .map_err(|err| {
            error!("Query user session err: {}", err);
            Error::new(ErrorKind::InvalidInput, "非法的 token")
        })?
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "token 不存在"))?;
    if session.expired_at < Utc::now() {
        return Err(Error::new(ErrorKind::InvalidInput, "已过期请重新登录"));
    }
    if session.source != SourceType::Third.as_i16() {
        return Err(Error::new(ErrorKind::InvalidInput, "暂不支持该渠道登录"));
    }

    Ok(session)
}

// 获取用户信息
async fn get_user_identity(db: &PgPool, user_id: i64) -> Result<UserIdentity, Error> {
    let user = UserIdentity::find_by_user_id(db, user_id)
        .await
        .map_err(|err| {
            error!("Query user identity err: {}", err);
            Error::new(ErrorKind::InvalidInput, "读取用户信息错误")
        })?
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "用户不存在"))?;

    // 非法用户不允许登录
    if user.status != StatusType::Active.as_i16() {
        return Err(Error::new(ErrorKind::InvalidInput, "该账户已被封禁"));
    }

    Ok(user)
}

// 登录
pub async fn login(
    app_conf: web::Data<AppConfig>,
    req: UserLoginReq,
) -> Result<UserInfoResp, Error> {
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

    Ok(UserInfoResp {
        user_id: user.id.unwrap_or_default(),
        username: user.provider_username,
        email: user.provider_email,
        role: user.role,
        status: user.status,
    })
}

// 获取用户信息
pub async fn info(app_conf: web::Data<AppConfig>, token: &str) -> Result<UserInfoResp, Error> {
    let db = &app_conf.db;

    // session 信息
    let session = get_user_session(db, token, TokenType::Login.as_i16()).await?;

    // 用户信息
    let user = get_user_identity(db, session.user_id).await?;

    Ok(UserInfoResp {
        user_id: user.id.unwrap_or_default(),
        username: user.provider_username,
        email: user.provider_email,
        role: user.role,
        status: user.status,
    })
}
