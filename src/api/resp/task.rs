use crate::enums::task::TaskStatus;
use crate::model::task::Task;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfoResp {
    pub id: i64,
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub author: String,
    pub email: String,
    pub status: i16,
    pub status_desc: String,
    pub result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Task> for TaskInfoResp {
    fn from(row: Task) -> Self {
        Self {
            id: row.id,
            question_cate_id: row.question_cate_id,
            task_type: 0,
            name: row.name.clone(),
            author: "".to_string(),
            status: row.status,
            status_desc: TaskStatus::desc(row.status).to_string(),
            email: row.email.clone(),
            result: row.result.clone(),
            created_at: to_local_datetime(Some(row.created_at)),
            updated_at: to_local_datetime(Some(row.updated_at)),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListResp {
    pub list: Vec<TaskInfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}
