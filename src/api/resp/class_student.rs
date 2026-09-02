use crate::enums::student::StudentStatus;
use crate::model::class_student::ClassStudent;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
pub struct ClassStudentResp {
    pub id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub account: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub remark: String,
    #[serde(rename(serialize = "lastLoginTime"))]
    pub last_login_time: String,
    #[serde(rename(serialize = "loginCount"))]
    pub login_count: i64,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

impl From<ClassStudent> for ClassStudentResp {
    fn from(raw: ClassStudent) -> Self {
        Self {
            id: raw.id,
            class_id: raw.class_id,
            user_id: raw.user_id,
            account: raw.account,
            status: raw.status,
            status_desc: StudentStatus::desc(raw.status),
            remark: raw.remark,
            last_login_time: if raw.last_login_time.is_none() {
                "".to_string()
            } else {
                to_local_datetime(raw.last_login_time.unwrap_or_default())
            },
            login_count: raw.login_count,
            created_at: to_local_datetime(raw.created_at.unwrap_or_default()),
            updated_at: to_local_datetime(raw.updated_at.unwrap_or_default()),
        }
    }
}
