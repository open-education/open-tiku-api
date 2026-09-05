use serde::Deserialize;

#[derive(Deserialize)]
pub struct TextbookDictInfo {
    pub id: i32,
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(deserialize = "typeCode",))]
    pub type_code: String,
    #[serde(rename(deserialize = "itemValue"))]
    pub item_value: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "isSelect"))]
    pub is_select: bool,
}

// 题目解析工具
#[derive(Deserialize)]
pub struct QuestionSnippetReq {
    #[serde(rename(deserialize = "typeList"))]
    pub type_list: Vec<TextbookDictInfo>,
    #[serde(rename(deserialize = "tagList"))]
    pub tag_list: Vec<TextbookDictInfo>,
    pub content: String,
}
