use crate::enums::user::{ProviderType, RoleType, StatusType};
use crate::model::user_identity::UserIdentity;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentityInfoResp {
    pub id: i64,
    pub user_id: i64,
    pub provider: i16,
    pub provider_desc: String,
    pub provider_username: String,
    pub provider_email: String,
    pub last_login_time: String,
    pub login_count: i64,
    pub role: i16,
    pub role_desc: String,
    pub status: i16,
    pub status_desc: String,
    pub remark: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserIdentity> for UserIdentityInfoResp {
    fn from(row: UserIdentity) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            user_id: row.user_id,
            provider: row.provider,
            provider_desc: ProviderType::desc(row.provider).to_string(),
            provider_username: row.provider_username.unwrap_or_default(),
            provider_email: row.provider_email.unwrap_or_default(),
            last_login_time: to_local_datetime(row.last_login_time),
            login_count: row.login_count,
            role: row.role,
            role_desc: RoleType::desc(row.role).to_string(),
            status: row.status,
            status_desc: StatusType::desc(row.status).to_string(),
            remark: row.remark,
            created_at: to_local_datetime(row.created_at),
            updated_at: to_local_datetime(row.updated_at),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListResp {
    pub list: Vec<UserIdentityInfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSessionInfoResp {
    pub id: i64,
    pub user_id: i64,
    pub source_desc: String,
    pub username: String,
    pub provider_desc: String,
    pub expired_at: String,
    pub renew_cnt: i16,
    pub client_ip: String,
    pub user_agent: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSessionListResp {
    pub list: Vec<UserSessionInfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}
