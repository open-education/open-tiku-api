use crate::enums::task::TaskStatus;
use crate::model::task::Task;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
pub struct TaskInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(serialize = "taskType"))]
    pub task_type: i16,
    pub name: String,
    pub author: String,
    pub email: String,
    pub status: i16,
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub result: Option<String>,
    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
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
pub struct TaskListResp {
    pub list: Vec<TaskInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
