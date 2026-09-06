use crate::enums::student::StudentStatus;
use crate::model::class_student::ClassStudent;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassStudentResp {
    pub id: i64,
    pub class_id: i64,
    pub user_id: i64,
    pub account: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    pub status_desc: String,
    pub remark: String,
    pub last_login_time: String,
    pub login_count: i64,
    pub created_at: String,
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
            status_desc: StudentStatus::desc(row.status).to_string(),
            remark: row.remark,
            last_login_time: to_local_datetime(row.last_login_time),
            login_count: row.login_count,
            created_at: to_local_datetime(row.created_at),
            updated_at: to_local_datetime(row.updated_at),
        }
    }
}
