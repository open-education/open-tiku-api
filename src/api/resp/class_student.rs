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
    fn from(row: ClassStudent) -> Self {
        Self {
            id: row.id,
            class_id: row.class_id,
            user_id: row.user_id,
            account: row.account,
            status: row.status,
            status_desc: StudentStatus::desc(row.status),
            remark: row.remark,
            last_login_time: if row.last_login_time.is_none() {
                "".to_string()
            } else {
                to_local_datetime(row.last_login_time.unwrap_or_default())
            },
            login_count: row.login_count,
            created_at: to_local_datetime(row.created_at.unwrap_or_default()),
            updated_at: to_local_datetime(row.updated_at.unwrap_or_default()),
        }
    }
}
