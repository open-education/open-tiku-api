use crate::AppConfig;
use crate::model::user_session::UserSession;
use log::{error, info};

// 删除过期的 sessions

pub async fn cleanup(conf: &AppConfig) {
    match UserSession::delete_expired_sessions(&conf.db).await {
        Ok(rows) => {
            info!("Deleting expired sessions: {:?}", rows);
        }
        Err(e) => {
            error!("Error while deleting expired sessions: {}", e)
        }
    }
}
