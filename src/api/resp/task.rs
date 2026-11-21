use crate::enums::task::TaskStatus;
use crate::model::task::Task;
use crate::util::local::to_local_datetime;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfoResp {
    pub id: i64,
    pub question_cate_id: i64,
    pub chapter_id: i32,        // 末级章节标识
    pub chapter_path: String,   // 末级章节完整父路径
    pub knowledge_id: i32,      // 末级知识点标识
    pub knowledge_path: String, // 末级知识点完整父路径
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
            chapter_id: raw.chapter_id,
            chapter_path: raw.chapter_path,
            knowledge_id: raw.knowledge_id,
            knowledge_path: raw.knowledge_path,
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
