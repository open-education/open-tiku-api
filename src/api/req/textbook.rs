use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTextbookReq {
    pub id: Option<i32>,
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<i32>,
    pub label: String,
    #[serde(rename(deserialize = "pathDepth"))]
    pub path_depth: Option<i32>,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "pathType"))]
    pub path_type: Option<String>,
    pub path: String,
}
