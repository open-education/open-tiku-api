use serde::Deserialize;

// 题目解析工具
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionSnippetReq {
    pub textbook_id: i32,
    pub content: String,
}
