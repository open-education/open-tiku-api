use crate::app::config::AppState;
use crate::constant::meta;
use crate::enums::user::RoleType;
use crate::model::user_session::{UserSession, UserSource};
use crate::service::class_student::get_student_by_user_id;
use crate::service::user_identity::get_user_identity_by_user_id;
use crate::service::user_session::get_user_session_by_token;
use actix_web::dev::{Payload, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header::USER_AGENT;
use actix_web::middleware::Next;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, web};
use chrono::{Duration, Utc};
use log::error;
use serde::Serialize;
use std::future::{Ready, ready};

// 三方普通登录用户信息验证
#[derive(Serialize, Clone)]
pub struct UserInfo {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: i16,
    pub status: i16,
    // 不需要处理可见性, 前端随便就能看到, 也是公开可查看的值
    pub token: Option<String>,
}

// 定义一个提取器，用于在 Handler 中方便地获取 UserInfo, 不存在时返回 401
// 对于可选的则使用 Option<UserInfo> 接收, 框架已经实现, 不存在返回 None
impl FromRequest for UserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user_info = req.extensions().get::<UserInfo>().cloned();
        match user_info {
            Some(info) => ready(Ok(info)),
            None => ready(Err(ErrorUnauthorized("Unauthorized"))),
        }
    }
}

// 教师用户登录验证
#[derive(Serialize, Clone)]
pub struct TeacherUserInfo(pub UserInfo);

// 教师用户提取器
impl FromRequest for TeacherUserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user_info) = req.extensions().get::<UserInfo>() {
            if user_info.role == RoleType::Teacher.as_i16() {
                return ready(Ok(TeacherUserInfo(user_info.clone())));
            }
        }

        ready(Err(ErrorUnauthorized("权限不足, 仅限教师用户访问")))
    }
}

// 学生用户
#[derive(Serialize, Clone)]
pub struct StudentUserInfo(pub UserInfo);

impl FromRequest for StudentUserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user_info) = req.extensions().get::<UserInfo>() {
            if user_info.role == RoleType::Student.as_i16() {
                return ready(Ok(StudentUserInfo(user_info.clone())));
            }
        }

        ready(Err(ErrorUnauthorized("权限不足, 仅限学生用户访问")))
    }
}

// 客户端信息
pub struct ClientInfo {
    pub ip: String,
    pub user_agent: String,
}

impl FromRequest for ClientInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let client_ip = req
            .headers()
            .get("X-Real-IP")
            .and_then(|h| h.to_str().ok())
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| {
                req.connection_info()
                    .realip_remote_addr()
                    .unwrap_or("IP not available")
                    .to_string()
            });

        let user_agent = req
            .headers()
            .get(USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("User-Agent not provided")
            .to_string();

        ready(Ok(ClientInfo {
            ip: client_ip,
            user_agent,
        }))
    }
}

// 完全不需要认证的白名单路径, 配置时简单浏览下, 需要认证和不需要认证两种配置内容少的一类
// 内容比较少时直接数组即可, 内容多时再更新为 Set
const PREFIX_LIST: &[&str] = &[
    // 题目
    "/question/info/",
    "/question/similar",
    "/question/original",
    // 导航菜单
    "/textbook/list/",
    // 通过字典
    "/other/dict/list/",
    // 任务
    "/task/list",
    // 试卷
    "/paper/top/info/",
    "/paper/gen/info/",
    "/paper/latest/",
    // 用户
    "/user/exchange",
    "/user/login",
    // 回调
    "/callback/",
    // 文本工具
    "/text/question/snippet",
];

// 如果有登录信息时需要解析的白名单, 没有则不需要解析
const OPTION_PREFIX_LIST: &[&str] = &[
    // 题目
    "/question/list",
    // 试卷
    "/paper/list",
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
    if PREFIX_LIST.iter().any(|prefix| path.starts_with(prefix)) {
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

            let err = ErrorUnauthorized("Missing or invalid token");
            return Err((err, req));
        }
    };

    // 获取全局配置
    let app_state = match req.app_data::<web::Data<AppState>>() {
        Some(data) => data,
        None => {
            let err = actix_web::error::ErrorInternalServerError("服务配置参数错误");
            return Err((err, req));
        }
    };

    let db = &app_state.db;

    // 获取用户会话
    let mut session = match get_user_session_by_token(db, token).await {
        Ok(s) => s,
        Err(err) => {
            error!("Wrap get user session err: {}", err.msg);
            let err = actix_web::error::ErrorInternalServerError("获取用户登录信息错误");
            return Err((err, req));
        }
    };

    // 获取用户身份
    let user_info: UserInfo = if session.source == UserSource::User.as_i16() {
        match get_user_identity_by_user_id(db, session.user_id).await {
            Ok(user) => UserInfo {
                user_id: user.user_id,
                username: user.provider_username,
                email: user.provider_email,
                role: user.role,
                status: user.status,
                token: Some(token.to_string()),
            },
            Err(err) => {
                error!("Wrap get user identity err: {}", err.msg);
                let err = actix_web::error::ErrorInternalServerError("获取第三方用户身份信息错误");
                return Err((err, req));
            }
        }
    } else if session.source == UserSource::Student.as_i16() {
        match get_student_by_user_id(db, session.user_id).await {
            Ok(user) => UserInfo {
                user_id: user.user_id,
                username: Some(user.account),
                email: None,
                role: UserSource::Student.as_i16(),
                status: user.status,
                token: Some(token.to_string()),
            },
            Err(err) => {
                error!("Wrap get user identity err: {}", err.msg);
                let err = actix_web::error::ErrorInternalServerError("获取学生账户信息错误");
                return Err((err, req));
            }
        }
    } else {
        let err = ErrorUnauthorized("不支持的登录 token");
        return Err((err, req));
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
                let err = actix_web::error::ErrorInternalServerError("更新用户 Session 信息错误");
                return Err((err, req));
            }
        };
    }

    // 插入用户信息并返回
    req.extensions_mut().insert(user_info);

    Ok(req)
}
