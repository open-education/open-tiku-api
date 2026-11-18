use crate::enums::task::TaskStatus;
use crate::model::task::Task;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfoResp {
    pub id: i64,
    pub two_level_id: i32,
    pub five_level_id: i32,
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub author: String,
    pub email: String,
    pub status: i16,
    pub status_desc: String,
    pub result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Task> for TaskInfoResp {
    fn from(raw: Task) -> Self {
        Self {
            id: raw.id,
            two_level_id: raw.two_level_id,
            five_level_id: raw.five_level_id,
            question_cate_id: raw.question_cate_id,
            task_type: raw.task_type,
            name: raw.name,
            url: raw.url,
            author: String::new(),
            status: raw.status,
            status_desc: TaskStatus::desc(raw.status).to_string(),
            email: raw.email,
            result: raw.result,
            created_at: to_local_datetime(Some(raw.created_at)),
            updated_at: to_local_datetime(Some(raw.updated_at)),
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
