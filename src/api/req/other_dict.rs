use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTextbookDictReq {
    pub id: Option<i32>,
    #[serde(rename(deserialize = "textbookId"))]
    pub textbook_id: i32,
    #[serde(rename(deserialize = "typeCode"))]
    pub type_code: String,
    #[serde(rename(deserialize = "itemValue"))]
    pub item_value: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "isSelect"))]
    pub is_select: bool,
}
