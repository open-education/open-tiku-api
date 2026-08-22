use crate::api::user::{
    ExchangeTokenReq, UserEditReq, UserIdentityInfoResp, UserListReq, UserListResp, UserLoginReq,
    UserSessionInfoResp, UserSessionListReq, UserSessionListResp,
};
use crate::app::config::AppState;
use crate::constant::meta;
use crate::enums::user::RoleType;
use crate::middleware::user::{ClientInfo, UserInfo};
use crate::model::class_student::ClassStudent;
use crate::model::user_identity::{ProviderType, StatusType, UserIdentity};
use crate::model::user_session::{UserSession, UserSource};
use crate::service::class_student::get_student_by_user_id;
use crate::service::user_identity::get_user_identity_by_user_id;
use crate::service::user_session::get_user_session_by_token;
use crate::util::argon2::verify_password;
use crate::util::error::AppError;
use crate::util::local::to_local_datetime;
use crate::util::pwd::get_pwd;
use chrono::{Duration, Utc};
use reqwest::get;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

// 换取登录 token
pub async fn exchange(app_state: &AppState, req: ExchangeTokenReq) -> Result<String, AppError> {
    let db = &app_state.db;

    // session 信息
    let mut session = get_user_session_by_token(db, req.temp_token.as_str()).await?;

    // 只有第三方登录用户需要换取登录 token
    if session.source != UserSource::User.as_i16() {
        return Err(AppError::param_error("非法的交换 token"));
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
        AppError::db_error("更新用户 session 信息错误")
    })?;

    Ok(login_token)
}

// 登录
pub async fn login(
    app_state: &AppState,
    req: UserLoginReq,
    client_info: ClientInfo,
) -> Result<UserInfo, AppError> {
    // 检查登录来源是否支持
    let source =
        UserSource::from_i16(req.source).ok_or_else(|| AppError::param_error("非法的登录来源"))?;

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
                app_state.config.login.student_private_key_pem.as_str(),
            )
            .await
        }
        _ => Err(AppError::business_error("非法的登录类型")),
    }
}

// 第三方用户登录
async fn handle_normal_login(
    db: &PgPool,
    req: &UserLoginReq,
    client_info: &ClientInfo,
) -> Result<UserInfo, AppError> {
    let token: String = req.token.clone().unwrap_or_default();
    if token.is_empty() {
        return Err(AppError::param_error("登录信息不存在"));
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
        AppError::db_error("更新用户 session 信息错误")
    })?;

    // 更新用户统计信息
    user.last_login_time = Some(Utc::now());
    user.login_count = user.login_count + 1;
    let _ = UserIdentity::save(db, &user).await.map_err(|err| {
        error!("Login save user session save err: {}", err);
        AppError::db_error("更新用户信息错误")
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
    private_key_pem: &str,
) -> Result<UserInfo, AppError> {
    let account: String = req.account.clone().unwrap_or_default();
    let password: String = req.password.clone().unwrap_or_default();
    if account.is_empty() || password.is_empty() {
        return Err(AppError::param_error("登录信息不存在"));
    }

    // 用私钥解密密码
    let get_d_pwd = get_pwd(&password, private_key_pem)?;

    // 查询学生账户信息
    let mut student = ClassStudent::find_by_account(db, account.as_str())
        .await
        .map_err(|e| {
            error!("查询学生账户失败: {}", e);
            AppError::db_error("学生账户查询出错")
        })?
        .ok_or_else(|| AppError::not_found("学生账户不存在"))?;

    // 进行密码比对
    match verify_password(pepper, get_d_pwd.as_str(), student.password.as_str()) {
        Ok(_) => {}
        Err(_) => {
            return Err(AppError::business_error("用户名密码不匹配"));
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
        expired_at: Utc::now() + Duration::hours(meta::LOGIN_TOKEN_EXPIRED_HOUR),
        renew_cnt: 1,
        client_ip: client_info.ip.clone(),
        user_agent: client_info.user_agent.clone(),
        created_at: None,
        updated_at: None,
    };
    let _ = UserSession::save(db, session).await.map_err(|e| {
        error!("Save user session failed: {}", e);
        AppError::db_error("生成学生登录信息失败")
    })?;

    // 更新用户统计信息
    student.last_login_time = Some(Utc::now());
    student.login_count = student.login_count + 1;
    let _ = ClassStudent::update_by_id(db, &student)
        .await
        .map_err(|err| {
            error!("Update student user session student by id error: {}", err);
            AppError::db_error("更新学生用户统计信息错误")
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
pub async fn info(app_state: &AppState, token: &str) -> Result<UserInfo, AppError> {
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
        _ => Err(AppError::business_error("未知的用户来源")),
    }
}

// 退出登录
pub async fn logout(app_state: &AppState, user_info: UserInfo) -> Result<bool, AppError> {
    let db = &app_state.db;

    // session 信息
    let session =
        get_user_session_by_token(db, user_info.token.unwrap_or_default().as_str()).await?;

    UserSession::delete_by_id(db, session.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!("Login delete user session delete err: {}", err);
            AppError::db_error("清空 Session 失败")
        })?;

    Ok(true)
}

pub async fn get_user_map(
    db: &PgPool,
    author_ids: Vec<i64>,
) -> Result<HashMap<i64, String>, AppError> {
    let user_list = UserIdentity::find_by_user_ids(db, author_ids)
        .await
        .map_err(|e| {
            error!("user list by id err: {:?}", e);
            AppError::db_error("作者信息查询失败")
        })?;
    let user_map: HashMap<i64, String> = user_list
        .into_iter()
        .map(|user| (user.user_id, user.provider_username.unwrap_or_default()))
        .collect();

    Ok(user_map)
}

// 第三方账户列表
pub async fn account_list(
    app_state: &AppState,
    req: UserListReq,
) -> Result<UserListResp, AppError> {
    let db = &app_state.db;

    let count = UserIdentity::count(db).await.map_err(|e| {
        error!("list user count err: {}", e);
        AppError::db_error("用户计数查询失败")
    })?;

    let offset = (req.page_no - 1) * req.page_size;
    if offset >= count as i32 {
        return Ok(UserListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total: count,
        });
    }

    let rows = UserIdentity::list(db, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("list user list rows err: {}", e);
            AppError::db_error("用户列表查询失败")
        })?;

    Ok(UserListResp {
        list: rows.into_iter().map(to_user_identity_info_resp).collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total: count,
    })
}

fn to_user_identity_info_resp(raw: UserIdentity) -> UserIdentityInfoResp {
    UserIdentityInfoResp {
        id: raw.id.unwrap_or_default(),
        user_id: raw.user_id,
        provider: raw.provider,
        provider_desc: ProviderType::desc(raw.provider),
        provider_username: raw.provider_username.unwrap_or_default(),
        provider_email: raw.provider_email.unwrap_or_default(),
        last_login_time: if raw.last_login_time.is_some() {
            to_local_datetime(raw.last_login_time.unwrap_or_default())
        } else {
            "".to_string()
        },
        login_count: raw.login_count,
        role: raw.role,
        role_desc: RoleType::desc(raw.role),
        status: raw.status,
        status_desc: StatusType::desc(raw.status),
        remark: raw.remark,
        created_at: to_local_datetime(raw.created_at.unwrap_or_default()),
        updated_at: to_local_datetime(raw.updated_at.unwrap_or_default()),
    }
}

// session 列表
pub async fn session_list(
    app_state: &AppState,
    req: UserSessionListReq,
) -> Result<UserSessionListResp, AppError> {
    let db = &app_state.db;

    let count = UserSession::count(db).await.map_err(|e| {
        error!("list user count err: {}", e);
        AppError::db_error("用户 Session 计数查询失败")
    })?;

    let offset = (req.page_no - 1) * req.page_size;
    if offset >= count as i32 {
        return Ok(UserSessionListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total: 0,
        });
    }

    let rows = UserSession::list(db, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("list user list rows err: {}", e);
            AppError::db_error("用户 Session 列表查询失败")
        })?;

    // 根据来源获取两份用户信息
    let mut account_ids: Vec<i64> = vec![];
    let mut student_ids: Vec<i64> = vec![];
    for row in &rows {
        if row.source == UserSource::User.as_i16() {
            account_ids.push(row.user_id);
        } else {
            student_ids.push(row.user_id);
        }
    }

    let mut account_map: HashMap<i64, UserIdentity> = HashMap::new();
    if !account_ids.is_empty() {
        let account_list = UserIdentity::find_by_user_ids(db, account_ids)
            .await
            .map_err(|e| {
                error!("list user list err: {}", e);
                AppError::db_error("用户列表查询失败")
            })?;
        account_map = account_list
            .into_iter()
            .map(|item| (item.user_id, item))
            .collect();
    }

    let mut student_map: HashMap<i64, ClassStudent> = HashMap::new();
    if !student_ids.is_empty() {
        let student_list = ClassStudent::find_by_user_ids(db, student_ids)
            .await
            .map_err(|e| {
                error!("list class student list err: {}", e);
                AppError::db_error("学生账户列表查询失败")
            })?;
        student_map = student_list
            .into_iter()
            .map(|item| (item.user_id, item))
            .collect();
    }

    Ok(UserSessionListResp {
        list: to_session_info_resp(rows, account_map, student_map),
        page_no: req.page_no,
        page_size: req.page_size,
        total: count,
    })
}

fn to_session_info_resp(
    rows: Vec<UserSession>,
    account_map: HashMap<i64, UserIdentity>,
    student_map: HashMap<i64, ClassStudent>,
) -> Vec<UserSessionInfoResp> {
    let mut resp_list: Vec<UserSessionInfoResp> = vec![];
    for row in rows.into_iter() {
        let mut username: String = "".to_string();
        let mut provider_desc: String = "".to_string();

        if row.source == UserSource::User.as_i16() {
            if let Some(account) = account_map.get(&row.user_id) {
                username = account
                    .provider_username
                    .clone()
                    .unwrap_or("未知".to_string());
                provider_desc = ProviderType::desc(account.provider);
            }
        } else {
            if let Some(student) = student_map.get(&row.user_id) {
                username = student.account.clone();
                provider_desc = "班级".to_string();
            }
        }

        resp_list.push(UserSessionInfoResp {
            id: row.id.unwrap_or_default(),
            user_id: row.user_id,
            source_desc: UserSource::desc(row.source),
            username: username.clone(),
            provider_desc: provider_desc.clone(),
            expired_at: to_local_datetime(row.expired_at),
            renew_cnt: row.renew_cnt,
            client_ip: row.client_ip.clone(),
            user_agent: row.user_agent.clone(),
            created_at: to_local_datetime(row.created_at.unwrap_or_default()),
            updated_at: to_local_datetime(row.updated_at.unwrap_or_default()),
        })
    }

    resp_list
}

pub async fn edit(app_state: &AppState, req: UserEditReq) -> Result<bool, AppError> {
    StatusType::from_i16(req.status).ok_or_else(|| AppError::param_error("用户状态错误"))?;

    let rows = UserIdentity::update_by_id(&app_state.db, req)
        .await
        .map_err(|e| {
            error!("edit user identity err: {}", e);
            AppError::db_error("用户状态更新失败")
        })?;

    // 清除该用户的登录信息, 访问中间件获取用户 session 和 info 时会验证, 该处不需要重复该逻辑

    Ok(rows)
}
