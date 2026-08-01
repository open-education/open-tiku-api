use crate::AppConfig;
use crate::api::callback::CallbackQuery;

use crate::constant::meta;
use crate::model::user_identity::{ProviderType, RoleType, StatusType, UserIdentity};
use crate::model::user_session::UserSession;
use crate::util::snowflake;
use actix_web::{Error, HttpResponse, Result, error, web};
use chrono::{Duration, Utc};
use log::{error, info};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

// Github 登录
pub async fn github(app_conf: web::Data<AppConfig>, query: CallbackQuery) -> Result<HttpResponse> {
    let code = get_query_code(query, ProviderType::Github)?;

    let github_user = get_github_user(
        app_conf.github.0.as_str(),
        app_conf.github.1.as_str(),
        code.as_ref(),
    )
    .await?;

    let db = &app_conf.db;

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
            format!("{}/?token={}", app_conf.website_home_url, temp_token),
        ))
        .finish())
}

// QQ 登录
pub async fn qq(app_conf: web::Data<AppConfig>, query: CallbackQuery) -> Result<HttpResponse> {
    let code = get_query_code(query, ProviderType::QQ)?;

    let (open_id, qq_user) = get_qq_user(
        app_conf.qq.0.as_str(),
        app_conf.qq.1.as_str(),
        app_conf.qq.2.as_str(),
        code.as_ref(),
    )
    .await?;

    let db = &app_conf.db;

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
            format!("{}/?token={}", app_conf.website_home_url, temp_token),
        ))
        .finish())
}

// 解析 请求携带的 code
// 提取 code，缺失或为空时返回 400 错误
// 比如 github: http://127.0.0.1:8082/callback/github?code=9ca3d96cf1809fdba60b
// qq: http://127.0.0.1:8082/callback/github?code=9ca3d96cf1809fdba60b&state=tiku
fn get_query_code(query: CallbackQuery, provider_type: ProviderType) -> Result<String, Error> {
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
    if provider_type == ProviderType::Github {
        return Ok(code);
    }

    // qq 登录还存在 state, 目前该值由客户端传递, 故验证欠缺
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

    Ok(code)
}

#[derive(Deserialize)]
struct GithubAccessTokenResp {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
}

// 请求 github 换取用户信息
async fn get_github_user(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<GithubUser, Error> {
    let client = reqwest::Client::new();

    // 请求 access_token，将 reqwest 错误转为 InternalServerError
    let token_response: GithubAccessTokenResp = client
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
            error::ErrorInternalServerError("Failed to parse user info response")
        })?;

    info!(
        "GitHub user: id={}, login={}, name={:?} email={:?}",
        github_user.id, github_user.login, github_user.name, github_user.email
    );

    Ok(github_user)
}

#[derive(Deserialize)]
struct QQAccessTokenResp {
    access_token: String,
    expires_in: i64,
}

#[derive(Deserialize)]
struct QQOpenIdResp {
    openid: String,
}

#[derive(Debug, Deserialize)]
struct QQUser {
    ret: i64,
    msg: String,
    nickname: Option<String>,
}

async fn get_qq_user(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<(String, QQUser), Error> {
    let client = reqwest::Client::new();

    // 请求 access_token，将 reqwest 错误转为 InternalServerError
    let access_token_resp: QQAccessTokenResp = client
        .get(format!("https://graph.qq.com/oauth2.0/token?grant_type=authorization_code&client_id={}&client_secret={}&code={}&redirect_uri={}&fmt=json", client_id, client_secret, code, redirect_uri))
        .send()
        .await
        .map_err(|e| {
            error!("Request QQ access_token failed: {}", e);
            error::ErrorInternalServerError("Failed to request access token")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse QQ access_token response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse access token response")
        })?;

    info!(
        "Access token obtained: access_token={}, expires_in={}",
        access_token_resp.access_token, access_token_resp.expires_in
    );

    // 请求 openid，将 reqwest 错误转为 InternalServerError
    let open_id_resp: QQOpenIdResp = client
        .get(format!(
            "https://graph.qq.com/oauth2.0/me?access_token={}&fmt=json",
            access_token_resp.access_token
        ))
        .send()
        .await
        .map_err(|e| {
            error!("Request QQ open id failed: {}", e);
            error::ErrorInternalServerError("Failed to request open id")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse QQ open id response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse open id response")
        })?;

    // 请求 user，将 reqwest 错误转为 InternalServerError
    let user_resp: QQUser = client
        .get(format!(
            "https://graph.qq.com/user/get_user_info?access_token={}&oauth_consumer_key={}&openid={}&fmt=json",
            access_token_resp.access_token, client_id, open_id_resp.openid
        ))
        .send()
        .await
        .map_err(|e| {
            error!("Request QQ user failed: {}", e);
            error::ErrorInternalServerError("Failed to request user info")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse QQ user response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse user info response")
        })?;

    info!(
        "QQ user: ret={}, msg={}, nickname={:?}",
        user_resp.ret, user_resp.msg, user_resp.nickname
    );

    if user_resp.ret != 0 {
        return Err(error::ErrorInternalServerError(user_resp.msg));
    }

    Ok((open_id_resp.openid, user_resp))
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
        token: token.to_string(),
        expired_at: Utc::now() + Duration::minutes(meta::TEMP_TOKEN_EXPIRED_MINUTE),
        renew_cnt: 0,
        client_ip: "".to_string(),
        user_agent: "".to_string(),
    };

    let _ = UserSession::save(db, session).await.map_err(|e| {
        error!("Save user session failed: {}", e);
        error::ErrorInternalServerError("Failed to save user session")
    })?;

    Ok(())
}
