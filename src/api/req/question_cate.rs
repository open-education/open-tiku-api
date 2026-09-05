use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateQuestionCateReq {
    pub id: Option<i32>,
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    #[serde(rename(deserialize = "sortOrder"))]
    pub sort_order: i32,
}
