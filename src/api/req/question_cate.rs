use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestionCateReq {
    pub id: Option<i32>,
    pub related_id: i32,
    pub label: String,
    pub sort_order: i32,
}
