use crate::AppConfig;
use crate::api::callback::GitHubCallbackQuery;

use crate::constant::meta;
use crate::model::user_identity::{ProviderType, RoleType, StatusType, UserIdentity};
use crate::model::user_session::{SourceType, TokenType, UserSession};
use crate::util::snowflake;
use actix_web::{Error, HttpResponse, Result, error, web};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

// 定义 GitHub OAuth 响应结构
#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

// 定义 GitHub 用户信息结构
#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
}

pub async fn github(
    app_conf: web::Data<AppConfig>,
    query: GitHubCallbackQuery,
) -> Result<HttpResponse> {
    let code = get_github_code(query)?;

    let github_user = get_github_user(
        app_conf.github.0.as_str(),
        app_conf.github.1.as_str(),
        code.as_ref(),
    )
    .await?;

    let db = &app_conf.db;

    // 保存用户信息
    let user = save_user_identity(db, &github_user).await?;

    // 业务逻辑生成内部 token, 此时为临时替换登录用的 token
    let temp_token = Uuid::new_v4().to_string();

    // 生成用户临时 session 信息, 如果已经存在则直接替换为最新
    save_user_session(db, temp_token.as_str(), user.user_id).await?;

    // 重定向回前端重新请求换取登录的 token
    Ok(HttpResponse::Found()
        .append_header((
            "Location",
            format!("{}/?token={}", app_conf.website_home_url, temp_token),
        ))
        .finish())
}

// 解析 github 请求携带的 code
// 提取 code，缺失或为空时返回 400 错误
// http://127.0.0.1:8082/callback/github?code=9ca3d96cf1809fdba60b
fn get_github_code(query: GitHubCallbackQuery) -> Result<String, Error> {
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

    Ok(code)
}

// 请求 github 换取用户信息
async fn get_github_user(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<GithubUser, Error> {
    let client = reqwest::Client::new();

    // 请求 access_token，将 reqwest 错误转为 InternalServerError
    let token_response: AccessTokenResponse = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
        }))
        .send()
        .await
        .map_err(|e| {
            error!("Request GitHub access_token failed: {}", e);
            error::ErrorInternalServerError("Failed to request access token")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse GitHub access_token response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse access token response")
        })?;

    // type=bearer, scope=user:email
    info!(
        "Access token obtained: type={}, scope={}",
        token_response.token_type, token_response.scope
    );

    // 请求用户信息, 邮箱需要单独请求其它接口, 暂时不考虑获取用户邮箱
    let github_user: GithubUser = client
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            format!("Bearer {}", token_response.access_token),
        )
        .header("User-Agent", "MyActixApp/1.0")
        .send()
        .await
        .map_err(|e| {
            error!("Request GitHub user failed: {}", e);
            error::ErrorInternalServerError("Failed to request user info")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse GitHub user response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse user info")
        })?;

    // user: id=247191289, login=zhangguangxun1, name=None
    info!(
        "GitHub user: id={}, login={}, name={:?} email={:?}",
        github_user.id, github_user.login, github_user.name, github_user.email
    );

    Ok(github_user)
}

// 保存用户信息
async fn save_user_identity(db: &PgPool, user: &GithubUser) -> Result<UserIdentity, Error> {
    // 名称拼接
    let name = if let Some(name) = user.name.clone() {
        format!("{} <{}>", user.login, name)
    } else {
        user.login.clone()
    };

    let mut has_user = UserIdentity::find_by_provider(
        db,
        ProviderType::Github.as_i16(),
        user.id.to_string().as_str(),
    )
    .await
    .map_err(|e| {
        error!("Error finding user identity by id: {}", e);
        error::ErrorInternalServerError("Failed to user_identity check github user")
    })?
    .unwrap_or_else(|| UserIdentity {
        id: None,
        user_id: snowflake::generate_id(),
        provider: ProviderType::Github.as_i16(),
        provider_user_id: user.id.to_string(),
        provider_username: Some(name.clone()),
        provider_email: user.email.clone(),
        last_login_time: None,
        login_count: 0,
        role: RoleType::Normal.as_i16(),
        status: StatusType::Active.as_i16(),
        created_at: Default::default(),
        updated_at: Default::default(),
    });

    // 如果是修改数据则只覆盖三方平台字段
    if has_user.user_id > 0 {
        has_user.provider_username = Some(name);
        has_user.provider_email = user.email.clone();
    }

    let _ = UserIdentity::save(db, &has_user).await.map_err(|e| {
        error!("Failed to save user identity: {}", e);
        error::ErrorInternalServerError("Failed to save user identity")
    })?;

    Ok(has_user)
}

// 保存用户临时 session, 如果存在则直接替换
async fn save_user_session(db: &PgPool, token: &str, user_id: i64) -> Result<(), Error> {
    let session = UserSession::find_one_by_user(
        db,
        user_id,
        SourceType::Third.as_i16(),
        TokenType::Temp.as_i16(),
    )
    .await
    .map_err(|e| {
        error!("Error finding user session by id: {}", e);
        error::ErrorInternalServerError("Failed to find user session")
    })?
    .unwrap_or_else(|| UserSession {
        id: None,
        user_id,
        source: SourceType::Third.as_i16(),
        token_type: TokenType::Temp.as_i16(),
        token: token.to_string(),
        expired_at: Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE),
        renew_cnt: 0,
        use_cnt: 0,
        created_at: Default::default(),
        updated_at: Default::default(),
    });

    let _ = UserSession::save(db, session).await.map_err(|e| {
        error!("Save user session failed: {}", e);
        error::ErrorInternalServerError("Failed to save user session")
    })?;

    Ok(())
}
