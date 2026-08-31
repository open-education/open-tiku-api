use crate::model::user_identity::{ProviderType, StatusType, UserIdentity};
use crate::model::user_session::{UserSession, UserSource};
use crate::util::github::get_github_user;
use crate::util::qq::get_qq_user;
use crate::util::snowflake;
use actix_web::{Error, HttpResponse, Result, error};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing::error;
use url::Url;
use uuid::Uuid;

use crate::api::req::callback::CallbackQueryReq;
use crate::app::conf::AppState;
use crate::constant::meta;
use crate::enums::user::RoleType;
use crate::util::error::AppError;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::Sha256;
use std::io::ErrorKind;

// 生成 state
async fn generate_state(secret: &str) -> String {
    let timestamp = Utc::now().timestamp().to_string();

    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(timestamp.as_bytes());
    let hash = hasher.finalize();

    let sig = URL_SAFE_NO_PAD.encode(hash);

    format!("{}.{}", timestamp, sig)
}

// 验证 state
async fn verify_state(state: &str, secret: &str) -> Result<bool, std::io::Error> {
    let parts: Vec<&str> = state.split('.').collect();
    if parts.len() != 2 {
        return Ok(false);
    }

    let timestamp = parts[0];
    let signature = parts[1];

    let ts: i64 = timestamp
        .parse()
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("校验信息错误: {}", e)))?;

    // 检查是否在 5 分钟内
    if Utc::now().timestamp() - ts > 300 {
        return Ok(false);
    }

    // 重新计算签名
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(timestamp.as_bytes());
    let hash = hasher.finalize();
    let expected = URL_SAFE_NO_PAD.encode(hash);

    Ok(signature == expected)
}

// 获取第三方登录地址
pub async fn login_url(
    app_state: &AppState,
    provider: i16,
) -> std::result::Result<String, AppError> {
    let provider_type = ProviderType::from_i16(provider).ok_or_else(|| {
        error!("Failed to parse provider type from provider: {}", provider);
        AppError::param_error("不受支持的登录方式")
    })?;

    let state = generate_state(&app_state.config.login.oauth_state_secret).await;

    let (base, params): (&str, Vec<(&str, &str)>) = match provider_type {
        ProviderType::Github => (
            "https://github.com/login/oauth/authorize",
            vec![
                ("client_id", &app_state.config.login.github.client_id),
                ("redirect_uri", &app_state.config.login.github.redirect_uri),
                ("state", &state),
            ],
        ),
        ProviderType::QQ => (
            "https://graph.qq.com/oauth2.0/authorize",
            vec![
                ("response_type", "code"),
                ("client_id", &app_state.config.login.qq.client_id),
                ("redirect_uri", &app_state.config.login.qq.redirect_uri),
                ("state", &state),
                ("scope", "get_user_info"),
            ],
        ),
    };

    let mut url = Url::parse(base).map_err(|e| {
        error!("Parse base url error: {}", e);
        AppError::param_error("地址信息错误")
    })?;
    // extend_pairs 会自动进行 URL 编码
    url.query_pairs_mut().extend_pairs(params);
    Ok(url.to_string())
}

// Github 登录
pub async fn github(app_state: &AppState, query: CallbackQueryReq) -> Result<HttpResponse> {
    let code = get_query_code(query, &app_state.config.login.oauth_state_secret).await?;

    let github_user = get_github_user(
        &app_state.config.login.github.client_id,
        &app_state.config.login.github.client_secret,
        &code,
    )
    .await?;

    let db = &app_state.db;

    // 保存用户信息
    // 名称拼接
    let name = if let Some(name) = github_user.name.clone() {
        format!("{} <{}>", github_user.login, name)
    } else {
        github_user.login.clone()
    };
    let user = save_user_identity(
        db,
        ProviderType::Github,
        github_user.id.to_string().as_str(),
        Some(name),
        github_user.email,
    )
    .await?;

    // 业务逻辑生成内部 token, 此时为临时替换登录用的 token
    let temp_token = Uuid::new_v4().to_string();

    // 生成用户临时 session 信息, 如果已经存在则直接替换为最新
    save_user_session(db, temp_token.as_str(), user.user_id).await?;

    // 重定向回前端重新请求换取登录的 token
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "{}/?token={}",
                app_state.config.login.website_home_url, temp_token
            ),
        ))
        .finish())
}

// QQ 登录
pub async fn qq(app_state: &AppState, query: CallbackQueryReq) -> Result<HttpResponse> {
    let code = get_query_code(query, &app_state.config.login.oauth_state_secret).await?;

    let (open_id, qq_user) = get_qq_user(
        &app_state.config.login.qq.client_id,
        &app_state.config.login.qq.client_secret,
        &app_state.config.login.qq.redirect_uri,
        &code,
    )
    .await?;

    let db = &app_state.db;

    // 保存用户信息
    let user = save_user_identity(
        db,
        ProviderType::QQ,
        open_id.as_str(),
        qq_user.nickname,
        None,
    )
    .await?;

    // 业务逻辑生成内部 token, 此时为临时替换登录用的 token
    let temp_token = Uuid::new_v4().to_string();

    // 生成用户临时 session 信息, 如果已经存在则直接替换为最新
    save_user_session(db, temp_token.as_str(), user.user_id).await?;

    // 重定向回前端重新请求换取登录的 token
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!(
                "{}/?token={}",
                app_state.config.login.website_home_url, temp_token
            ),
        ))
        .finish())
}

// 解析 请求携带的 code
// 提取 code，缺失或为空时返回 400 错误
// 比如 github: http://127.0.0.1:8082/callback/github?code=9ca3d96cf1809fdba60b
// qq: http://127.0.0.1:8082/callback/github?code=9ca3d96cf1809fdba60b&state=tiku
async fn get_query_code(
    query: CallbackQueryReq,
    oauth_state_secret: &str,
) -> Result<String, Error> {
    let code = query
        .code
        .as_ref()
        .ok_or_else(|| {
            error!("Missing code query parameter");
            error::ErrorBadRequest("Query code is required")
        })?
        .to_owned();

    if code.is_empty() {
        error!("Empty code query parameter");
        return Err(error::ErrorBadRequest("Query code is empty"));
    }

    let state = query
        .state
        .as_ref()
        .ok_or_else(|| {
            error!("Missing state query parameter");
            error::ErrorBadRequest("Query state is required")
        })?
        .to_owned();
    if state.is_empty() {
        error!("Empty state query parameter");
        return Err(error::ErrorBadRequest("Query state is empty"));
    }

    // 校验 state 字段值
    if !verify_state(&state, oauth_state_secret).await? {
        return Err(error::ErrorBadRequest("校验失败, 请重新发起登录"));
    }

    Ok(code)
}

// 保存用户信息
async fn save_user_identity(
    db: &PgPool,
    provider_type: ProviderType,
    provider_user_id: &str,
    provider_username: Option<String>,
    email: Option<String>,
) -> Result<UserIdentity, Error> {
    let mut has_user = UserIdentity::find_by_provider(db, provider_type.as_i16(), provider_user_id)
        .await
        .map_err(|e| {
            error!("Error finding user identity by id: {}", e);
            error::ErrorInternalServerError("Failed to user_identity check github user")
        })?
        .unwrap_or_else(|| UserIdentity {
            id: None,
            user_id: snowflake::generate_id(),
            provider: provider_type.as_i16(),
            provider_user_id: provider_user_id.to_owned(),
            provider_username: provider_username.clone(),
            provider_email: email.clone(),
            last_login_time: None,
            login_count: 0,
            role: RoleType::Normal.as_i16(),
            status: StatusType::Active.as_i16(),
            remark: "".to_string(),
            created_at: None,
            updated_at: None,
        });

    // 如果是修改数据则只覆盖三方平台字段
    if has_user.user_id > 0 {
        has_user.provider_username = provider_username;
        has_user.provider_email = email;
    }

    let _ = UserIdentity::save(db, &has_user).await.map_err(|e| {
        error!("Failed to save user identity: {}", e);
        error::ErrorInternalServerError("Failed to save user identity")
    })?;

    Ok(has_user)
}

// 保存用户临时 session, 如果存在则直接替换
async fn save_user_session(db: &PgPool, token: &str, user_id: i64) -> Result<(), Error> {
    let session = UserSession {
        id: None,
        user_id,
        source: UserSource::User.as_i16(),
        token: token.to_string(),
        expired_at: Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE),
        renew_cnt: 0,
        client_ip: "".to_string(),
        user_agent: "".to_string(),
        created_at: None,
        updated_at: None,
    };

    let _ = UserSession::save(db, session).await.map_err(|e| {
        error!("Save user session failed: {}", e);
        error::ErrorInternalServerError("Failed to save user session")
    })?;

    Ok(())
}
