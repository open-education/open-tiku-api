use actix_web::{Error, error};
use log::{error, info};
use reqwest::Client;
use serde::Deserialize;

// 解析 github 账户信息

pub async fn get_github_user(
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> actix_web::Result<GithubUser, Error> {
    let client = Client::new();

    let access_token = get_access_token(&client, &client_id, client_secret, code).await?;

    let user = get_user(&client, &access_token).await?;

    Ok(user)
}

#[derive(Deserialize, Debug)]
struct GithubAccessTokenResp {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn get_access_token(
    client: &Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> actix_web::Result<String, Error> {
    let resp: GithubAccessTokenResp = client
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
            error!("Parse GitHub access_token failed: {}", e);
            error::ErrorInternalServerError("Failed to request access token")
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Parse GitHub access_token response failed: {}", e);
            error::ErrorInternalServerError("Failed to parse access token response")
        })?;

    info!("Parse GitHub Access token resp: {:?}", resp);

    if let Some(error_code) = resp.error {
        let msg = resp
            .error_description
            .unwrap_or_else(|| format!("获取 Access token 失败: {}", error_code));
        return Err(error::ErrorBadRequest(msg));
    }

    let access_token = resp
        .access_token
        .ok_or_else(|| error::ErrorBadRequest("Access token 为空"))?;

    Ok(access_token)
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

async fn get_user(client: &Client, access_token: &str) -> actix_web::Result<GithubUser, Error> {
    let resp: GithubUser = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
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

    info!("Parse GitHub user resp: {:?}", resp);

    Ok(resp)
}
