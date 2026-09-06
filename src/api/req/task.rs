use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAddReq {
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    pub textbook_id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListReq {
    pub question_cate_id: i64,
    pub task_type: i16,
    pub page_no: i32,
    pub page_size: i32,
}
