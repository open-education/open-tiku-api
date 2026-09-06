use crate::enums::user::{ProviderType, RoleType, StatusType};
use crate::model::user_identity::UserIdentity;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserIdentityInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub provider: i16,
    #[serde(rename(serialize = "providerDesc"))]
    pub provider_desc: String,
    #[serde(rename(serialize = "providerUsername"))]
    pub provider_username: String,
    #[serde(rename(serialize = "providerEmail"))]
    pub provider_email: String,
    #[serde(rename(serialize = "lastLoginTime"))]
    pub last_login_time: String,
    #[serde(rename(serialize = "loginCount"))]
    pub login_count: i64,
    pub role: i16,
    #[serde(rename(serialize = "roleDesc"))]
    pub role_desc: String,
    pub status: i16,
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub remark: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
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
pub struct UserListResp {
    pub list: Vec<UserIdentityInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[derive(Serialize)]
pub struct UserSessionInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    #[serde(rename(serialize = "sourceDesc"))]
    pub source_desc: String,
    pub username: String,
    #[serde(rename(serialize = "providerDesc"))]
    pub provider_desc: String,
    #[serde(rename(serialize = "expiredAt"))]
    pub expired_at: String,
    #[serde(rename(serialize = "renewCnt"))]
    pub renew_cnt: i16,
    #[serde(rename(serialize = "clientIp"))]
    pub client_ip: String,
    #[serde(rename(serialize = "userAgent"))]
    pub user_agent: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct UserSessionListResp {
    pub list: Vec<UserSessionInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
