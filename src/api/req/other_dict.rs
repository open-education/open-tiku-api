use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTextbookDictReq {
    pub id: Option<i32>,
    pub textbook_id: i32,
    pub type_code: String,
    pub item_value: String,
    pub sort_order: i32,
    pub is_select: bool,
}
