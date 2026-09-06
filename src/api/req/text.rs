use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextbookDictInfo {
    pub id: i32,
    pub textbook_id: i32,
    pub type_code: String,
    pub item_value: String,
    pub sort_order: i32,
    pub is_select: bool,
}

// 题目解析工具
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionSnippetReq {
    pub type_list: Vec<TextbookDictInfo>,
    pub tag_list: Vec<TextbookDictInfo>,
    pub content: String,
}
