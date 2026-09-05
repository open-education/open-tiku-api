use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaskAddReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(deserialize = "taskType"))]
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
}

#[derive(Deserialize)]
pub struct TaskListReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i64,
    #[serde(rename(deserialize = "taskType"))]
    pub task_type: i16,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}
