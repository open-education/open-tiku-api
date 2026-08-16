use crate::model::user_identity::{StatusType, UserIdentity};
use log::error;
use sqlx::PgPool;
use std::io::ErrorKind;

// 获取第三方用户信息
pub async fn get_user_identity_by_user_id(
    db: &PgPool,
    user_id: i64,
) -> Result<UserIdentity, std::io::Error> {
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
            if user.status == StatusType::Paused.as_i16() {
                "该账户已被暂停"
            } else if user.status == StatusType::Forbidden.as_i16() {
                "该账户已被封禁"
            } else {
                "该账户无法继续使用"
            },
        ));
    }

    Ok(user)
}
