use crate::model::user_session::UserSession;
use crate::util::error::AppError;
use chrono::Utc;
use log::error;
use sqlx::PgPool;

// 根据 token 获取用户 session 信息
pub async fn get_user_session_by_token(db: &PgPool, token: &str) -> Result<UserSession, AppError> {
    let session = UserSession::find_by_token(db, token)
        .await
        .map_err(|err| {
            error!("Query user session err: {}", err);
            AppError::db_error("非法的 token")
        })?
        .ok_or_else(|| AppError::not_found("token 不存在"))?;
    if session.expired_at < Utc::now() {
        // 删除过期的 session
        let _ = UserSession::delete_by_id(db, session.id.unwrap())
            .await
            .map_err(|err| {
                error!("Wrap delete user session err: {}", err);
                AppError::db_error("删除过期 token 错误")
            })?;

        return Err(AppError::business_error("已过期请重新登录"));
    }

    Ok(session)
}
