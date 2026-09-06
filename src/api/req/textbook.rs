use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTextbookReq {
    pub id: Option<i32>,
    pub parent_id: Option<i32>,
    pub label: String,
    pub path_depth: Option<i32>,
    pub sort_order: i32,
    pub path_type: Option<String>,
    pub path: String,
}
