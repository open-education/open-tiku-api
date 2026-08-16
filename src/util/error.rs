// 业务错误定义

#[derive(Debug)]
pub struct AppError {
    pub code: i32,
    pub msg: String,
}

impl AppError {
    pub fn new(code: i32, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
        }
    }

    // 服务本身的错误
    pub fn internal_error(msg: &str) -> Self {
        Self::new(500, msg)
    }

    // 数据库相关的错误
    pub fn db_error(msg: &str) -> Self {
        Self::new(5001, msg)
    }

    // 参数相关的错误
    pub fn param_error(msg: &str) -> Self {
        Self::new(400, msg)
    }

    // 业务逻辑错误
    pub fn business_error(msg: &str) -> Self {
        Self::new(6001, msg)
    }

    // 权限错误
    pub fn permission_denied(msg: &str) -> Self {
        Self::new(401, msg)
    }

    // 针对数据不存在的错误
    pub fn not_found(msg: &str) -> Self {
        Self::new(404, msg)
    }
}
