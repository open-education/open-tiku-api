use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAddReq {
    pub question_cate_id: i32,
    pub chapter_id: Option<i32>,        // 末级章节标识
    pub chapter_path: Option<String>,   // 末级章节完整父路径
    pub knowledge_id: Option<i32>,      // 末级知识点标识
    pub knowledge_path: Option<String>, // 末级知识点完整父路径
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListReq {
    pub chapter_path: Option<String>,
    pub knowledge_path: Option<String>,
    pub question_cate_id: Option<i64>,
    pub task_type: i16,
    pub page_no: i32,
    pub page_size: i32,
}
