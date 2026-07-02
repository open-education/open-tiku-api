use crate::AppConfig;
use crate::constant::meta;
use crate::model::user_identity::{StatusType, UserIdentity};
use crate::model::user_session::{SourceType, TokenType, UserSession};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage, web};
use chrono::{Duration, Utc};
use log::error;
use serde::Serialize;
use sqlx::PgPool;
use std::future::{Ready, ready};
use std::io::ErrorKind;

// 用户信息验证

#[derive(Serialize, Clone)]
pub struct UserInfo {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: i16,
    pub status: i16,
    #[serde(skip)] // 序列化和反序列化时都跳过
    pub token: Option<String>,
}

// 定义一个提取器，用于在 Handler 中方便地获取 UserInfo, 不存在时返回 401
// 对于可选的则使用 Option<UserInfo> 接收, 框架已经实现, 不存在返回 None
impl actix_web::FromRequest for UserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let user_info = req.extensions().get::<UserInfo>().cloned();
        match user_info {
            Some(info) => ready(Ok(info)),
            None => ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized"))),
        }
    }
}

// 完全不需要认证的白名单路径, 配置时简单浏览下, 需要认证和不需要认证两种配置内容少的一类
// 内容比较少时直接数组即可, 内容多时再更新为 Set
const PREFIX_LIST: &[&str] = &[
    // 题目
    "/question/info/",
    "/question/similar",
    // 导航菜单
    "/textbook/list/",
    // 通过字典
    "/other/dict/list/",
    // 任务
    "/task/list",
    // 试卷
    "/paper/info/",
    "/paper/list",
    "/paper/latest/",
    // 用户
    "/user/exchange",
    "/user/login",
    // 回调
    "/callback/github",
];

// 如果有登录信息时需要解析的白名单, 没有则不需要解析
const OPTION_PREFIX_LIST: &[&str] = &[
    // 题目
    "/question/list",
];

pub async fn auth(
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    let req = match validator(req).await {
        Ok(req) => req,
        Err((err, _)) => return Err(err),
    };
    next.call(req).await
}

async fn validator(req: ServiceRequest) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // 前缀匹配 只要路径以某个白名单前缀开头 就跳过认证
    let path = req.path();
    if PREFIX_LIST
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        return Ok(req);
    }

    // 从请求头中解析 token
    let auth_header = req.headers().get("Authorization");
    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(h) if h.starts_with("Bearer ") => h.trim_start_matches("Bearer ").trim(),
        _ => {
            // 如果是部分跳过则不继续处理
            if OPTION_PREFIX_LIST
                .iter()
                .any(|prefix| path.starts_with(prefix))
            {
                return Ok(req);
            }

            let err = actix_web::error::ErrorUnauthorized("Missing or invalid token");
            return Err((err, req));
        }
    };

    // 获取全局配置
    let app_conf = match req.app_data::<web::Data<AppConfig>>() {
        Some(data) => data,
        None => {
            let err = actix_web::error::ErrorInternalServerError("参数错误");
            return Err((err, req));
        }
    };

    let db = &app_conf.db;

    // 获取用户会话
    let mut session = match get_user_session(db, token, TokenType::Login.as_i16()).await {
        Ok(s) => s,
        Err(err) => {
            error!("Wrap get user session err: {}", err);
            let err = actix_web::error::ErrorInternalServerError(err);
            return Err((err, req));
        }
    };

    // 获取用户身份
    let user = match get_user_identity(db, session.user_id).await {
        Ok(u) => u,
        Err(err) => {
            error!("Wrap get user identity err: {}", err);
            let err = actix_web::error::ErrorInternalServerError(err);
            return Err((err, req));
        }
    };

    // 如果过期时间有效的用户则需要给用户续期
    let remain = session.expired_at - Utc::now();
    if remain.num_seconds() > 0 && remain.num_seconds() <= 3600 {
        session.expired_at = session.expired_at + Duration::hours(meta::RENEW_TOKEN_EXPIRED_HOUR);
        session.renew_cnt = session.renew_cnt + 1; // 续期次数累加
        let _ = match UserSession::save(db, session).await {
            Ok(u) => u,
            Err(err) => {
                error!("Wrap save user session err: {}", err);
                let err = actix_web::error::ErrorInternalServerError(err);
                return Err((err, req));
            }
        };
    }

    // 插入用户信息并返回
    req.extensions_mut().insert(UserInfo {
        user_id: user.user_id,
        username: user.provider_username,
        email: user.provider_email,
        role: user.role,
        status: user.status,
        token: Some(token.to_string()),
    });

    Ok(req)
}

// 根据 token 获取用户 session 信息
pub async fn get_user_session(
    db: &PgPool,
    token: &str,
    token_type: i16,
) -> Result<UserSession, std::io::Error> {
    let session = UserSession::find_one_by_token(db, token, token_type)
        .await
        .map_err(|err| {
            error!("Query user session err: {}", err);
            std::io::Error::new(ErrorKind::InvalidInput, "非法的 token")
        })?
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "token 不存在"))?;
    if session.expired_at < Utc::now() {
        // 删除过期的 session
        let _ = UserSession::delete_by_id(db, session.id.unwrap())
            .await
            .map_err(|err| {
                error!("Wrap delete user session err: {}", err);
                std::io::Error::new(ErrorKind::InvalidInput, "删除过期 token 错误")
            })?;

        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "已过期请重新登录",
        ));
    }
    if session.source != SourceType::Third.as_i16() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "暂不支持该渠道登录",
        ));
    }

    Ok(session)
}

// 获取用户信息
pub async fn get_user_identity(db: &PgPool, user_id: i64) -> Result<UserIdentity, std::io::Error> {
    let user = UserIdentity::find_by_user_id(db, user_id)
        .await
        .map_err(|err| {
            error!("Query user identity err: {}", err);
            std::io::Error::new(ErrorKind::InvalidInput, "读取用户信息错误")
        })?
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "用户不存在"))?;

    // 非法用户不允许登录
    if user.status != StatusType::Active.as_i16() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "该账户已被封禁",
        ));
    }

    Ok(user)
}
