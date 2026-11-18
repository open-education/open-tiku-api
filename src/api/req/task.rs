use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAddReq {
    pub two_level_id: i32,
    pub five_level_id: i32,
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListReq {
    pub two_level_id: i32,
    pub five_level_id: Option<i32>,
    pub question_cate_id: Option<i64>,
    pub task_type: i16,
    pub page_no: i32,
    pub page_size: i32,
}
