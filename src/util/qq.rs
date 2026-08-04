use actix_web::{Error, error};
use log::{error, info};
use reqwest::Client;
use serde::Deserialize;

// qq 登录账户
// 网站应用接入流程: [https://wiki.connect.qq.com/网站应用接入流程]
// 错误码具体信息查看: [https://wiki.connect.qq.com/公共返回码说明]
// qq 登录的代码比较古老, 通常只有失败时才会返回 error 字段, 成功时一般没有该字段, 所以解析时需要特别处理返回结果
// 错误码 error 为 0 时成功, 其它错误码为失败, 失败信息在 error_description 中

pub async fn get_qq_user(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> actix_web::Result<(String, QQUser), Error> {
    let client = Client::new();

    let access_token =
        get_access_token(&client, client_id, client_secret, redirect_uri, code).await?;

    let openid = get_openid(&client, &access_token).await?;

    let user = get_user(&client, client_id, &access_token, &openid).await?;

    Ok((openid, user))
}

#[derive(Deserialize, Debug)]
struct QQAccessTokenResp {
    access_token: Option<String>,
    #[serde(default)]
    error: Option<i64>,
    #[serde(default)]
    error_description: Option<String>,
}

async fn get_access_token(
    client: &Client,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> actix_web::Result<String, Error> {
    let resp: QQAccessTokenResp = client
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

    info!("Parse QQ access token resp {:?}", resp);

    if let Some(error_code) = resp.error.filter(|&code| code != 0) {
        let msg = resp
            .error_description
            .unwrap_or_else(|| format!("获取 Access token 失败: {}", error_code));
        return Err(error::ErrorBadRequest(msg));
    }

    let access_token = resp
        .access_token
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error::ErrorBadRequest("Access token 为空"))?;

    Ok(access_token)
}

#[derive(Deserialize, Debug)]
struct QQOpenIdResp {
    openid: Option<String>,
    #[serde(default)]
    error: Option<i64>,
    #[serde(default)]
    error_description: Option<String>,
}

async fn get_openid(client: &Client, access_token: &str) -> actix_web::Result<String, Error> {
    let resp: QQOpenIdResp = client
        .get(format!(
            "https://graph.qq.com/oauth2.0/me?access_token={}&fmt=json",
            access_token
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

    info!("Parse QQ OpenID resp {:?}", resp);

    if let Some(error_code) = resp.error.filter(|&code| code != 0) {
        let msg = resp
            .error_description
            .unwrap_or_else(|| format!("获取 用户OpenID 失败: {}", error_code));
        return Err(error::ErrorBadRequest(msg));
    }

    let openid = resp
        .openid
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error::ErrorBadRequest("用户OpenID 为空"))?;

    Ok(openid)
}

#[derive(Debug, Deserialize)]
pub struct QQUser {
    pub ret: i64,
    pub msg: String,
    pub nickname: Option<String>,
}

async fn get_user(
    client: &Client,
    client_id: &str,
    access_token: &str,
    openid: &str,
) -> actix_web::Result<QQUser, Error> {
    let resp: QQUser = client
        .get(format!(
            "https://graph.qq.com/user/get_user_info?access_token={}&oauth_consumer_key={}&openid={}&fmt=json",
            access_token, client_id, openid
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

    info!("QQ get user info: resp: {:?}", resp);

    if resp.ret != 0 {
        return Err(error::ErrorBadRequest(resp.msg));
    }

    Ok(resp)
}
