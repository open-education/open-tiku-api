use crate::api::user::{ExchangeTokenReq, UserLoginReq};
use crate::app::config::AppState;
use crate::constant::meta;
use crate::middleware::user::{ClientInfo, UserInfo};
use crate::model::class_student::ClassStudent;
use crate::model::user_identity::{RoleType, UserIdentity};
use crate::model::user_session::{UserSession, UserSource};
use crate::service::class_student::get_student_by_user_id;
use crate::service::user_identity::get_user_identity_by_user_id;
use crate::service::user_session::get_user_session_by_token;
use crate::util::argon2::verify_password;
use actix_web::web;
use chrono::{Duration, Utc};
use log::error;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use uuid::Uuid;

// 换取登录 token
pub async fn exchange(
    app_state: web::Data<AppState>,
    req: ExchangeTokenReq,
) -> Result<String, Error> {
    let db = &app_state.db;

    // session 信息
    let mut session = get_user_session_by_token(db, req.temp_token.as_str()).await?;

    // 只有第三方登录用户需要换取登录 token
    if session.source != UserSource::User.as_i16() {
        return Err(Error::new(ErrorKind::InvalidInput, "非法的交换 token"));
    }

    // 用户信息
    let _ = get_user_identity_by_user_id(db, session.user_id).await?;

    // 替换 session 为登录 token
    let login_token = Uuid::new_v4().to_string();
    session.token = login_token.clone();
    session.expired_at = Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE);

    // 替换用户临时 session 为 登录 session
    let _ = UserSession::save(db, session).await.map_err(|err| {
        error!("Exchange save user session save err: {}", err);
        Error::new(ErrorKind::Other, "更新用户 session 信息错误")
    })?;

    Ok(login_token)
}

// 登录
pub async fn login(
    app_state: web::Data<AppState>,
    req: UserLoginReq,
    client_info: ClientInfo,
) -> Result<UserInfo, Error> {
    // 检查登录来源是否支持
    let source = UserSource::from_i16(req.source)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "非法的登录来源"))?;

    let db = &app_state.db;

    // 根据来源进行登录
    match source {
        s if s == UserSource::User => handle_normal_login(db, &req, &client_info).await,
        s if s == UserSource::Student => {
            handle_student_login(
                db,
                &req,
                &client_info,
                app_state.config.login.student_pepper.as_str(),
            )
            .await
        }
        _ => Err(Error::new(ErrorKind::InvalidInput, "非法的登录类型")),
    }
}

// 第三方用户登录
async fn handle_normal_login(
    db: &PgPool,
    req: &UserLoginReq,
    client_info: &ClientInfo,
) -> Result<UserInfo, Error> {
    let token: String = req.token.clone().unwrap_or_default();
    if token.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "登录信息不存在"));
    }

    // session 信息
    let mut session = get_user_session_by_token(db, token.as_str()).await?;

    // 用户信息
    let mut user = get_user_identity_by_user_id(db, session.user_id).await?;

    // 更新 session
    session.expired_at = Utc::now() + Duration::hours(meta::LOGIN_TOKEN_EXPIRED_HOUR);
    session.renew_cnt = session.renew_cnt + 1;
    // 实际登录时才填写用户跟踪信息
    session.client_ip = client_info.ip.clone();
    session.user_agent = client_info.user_agent.clone();

    // 更新用户 session
    let _ = UserSession::save(db, session).await.map_err(|err| {
        error!("Login save user session save err: {}", err);
        Error::new(ErrorKind::Other, "更新用户 session 信息错误")
    })?;

    // 更新用户统计信息
    user.last_login_time = Some(Utc::now());
    user.login_count = user.login_count + 1;
    let _ = UserIdentity::save(db, &user).await.map_err(|err| {
        error!("Login save user session save err: {}", err);
        Error::new(ErrorKind::Other, "更新用户信息错误")
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

// 学生账户登录
async fn handle_student_login(
    db: &PgPool,
    req: &UserLoginReq,
    client_info: &ClientInfo,
    pepper: &str,
) -> Result<UserInfo, Error> {
    let account: String = req.account.clone().unwrap_or_default();
    let password: String = req.password.clone().unwrap_or_default();
    if account.is_empty() || password.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "登录信息不存在"));
    }

    // 查询学生账户信息
    let mut student = ClassStudent::find_by_account(db, account.as_str())
        .await
        .map_err(|e| {
            error!("查询学生账户失败: {}", e);
            Error::new(ErrorKind::Other, "学生账户查询出错")
        })?
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "学生账户不存在"))?;

    // 进行密码比对
    match verify_password(pepper, password.as_str(), student.password.as_str()) {
        Ok(_) => {}
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "用户名密码不匹配"));
        }
    }

    // 登录的 token 信息
    let token: String = Uuid::new_v4().to_string();

    // 新增 session
    let session = UserSession {
        id: None,
        user_id: student.user_id,
        source: UserSource::Student.as_i16(),
        token: token.clone(),
        expired_at: Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE),
        renew_cnt: 1,
        client_ip: client_info.ip.clone(),
        user_agent: client_info.user_agent.clone(),
    };
    let _ = UserSession::save(db, session).await.map_err(|e| {
        error!("Save user session failed: {}", e);
        Error::new(ErrorKind::Other, "生成学生登录信息失败")
    })?;

    // 更新用户统计信息
    student.last_login_time = Some(Utc::now());
    student.login_count = student.login_count + 1;
    let _ = ClassStudent::update_by_id(db, &student)
        .await
        .map_err(|err| {
            error!("Update student user session student by id error: {}", err);
            Error::new(ErrorKind::Other, "更新学生用户统计信息错误")
        })?;

    Ok(UserInfo {
        user_id: student.user_id,
        username: Some(student.account),
        email: None,
        role: RoleType::Student.as_i16(),
        status: student.status,
        token: Some(token),
    })
}

// 获取用户信息
pub async fn info(app_state: web::Data<AppState>, token: &str) -> Result<UserInfo, Error> {
    let db = &app_state.db;

    let session = get_user_session_by_token(db, token).await?;

    // 根据用户来源分别处理
    match session.source {
        src if src == UserSource::User.as_i16() => {
            let user = get_user_identity_by_user_id(db, session.user_id).await?;

            Ok(UserInfo {
                user_id: user.user_id,
                username: user.provider_username,
                email: user.provider_email,
                role: user.role,
                status: user.status,
                token: None,
            })
        }
        src if src == UserSource::Student.as_i16() => {
            let student = get_student_by_user_id(db, session.user_id).await?;

            Ok(UserInfo {
                user_id: student.user_id,
                username: Some(student.account),
                email: None,
                role: RoleType::Student.as_i16(),
                status: student.status,
                token: None,
            })
        }
        _ => Err(Error::new(ErrorKind::InvalidInput, "未知的用户来源")),
    }
}

// 退出登录
pub async fn logout(app_state: web::Data<AppState>, user_info: UserInfo) -> Result<bool, Error> {
    let db = &app_state.db;

    // session 信息
    let session =
        get_user_session_by_token(db, user_info.token.unwrap_or_default().as_str()).await?;

    UserSession::delete_by_id(db, session.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!("Login delete user session delete err: {}", err);
            Error::new(ErrorKind::Other, " 清空 Session 失败")
        })?;

    Ok(true)
}

// 获取用户名, 获取不到返回 未知
pub async fn get_user_name(db: &PgPool, user_id: i64) -> String {
    match UserIdentity::find_by_user_id(db, user_id).await {
        Ok(Some(user)) => user.provider_username.unwrap_or_default(),
        Ok(None) => "未知".to_string(),
        Err(err) => {
            error!("get user name user id err: {}", err);
            "未知".to_string()
        }
    }
}

pub async fn get_user_map(
    db: &PgPool,
    author_ids: Vec<i64>,
) -> Result<HashMap<i64, String>, Error> {
    let user_list = UserIdentity::find_by_user_ids(db, &author_ids)
        .await
        .map_err(|e| {
            error!("user list by id err: {:?}", e);
            Error::new(ErrorKind::Other, "作者信息查询失败")
        })?;
    let user_map: HashMap<i64, String> = user_list
        .into_iter()
        .map(|user| (user.user_id, user.provider_username.unwrap_or_default()))
        .collect();

    Ok(user_map)
}
