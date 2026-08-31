use crate::model::question_cate::QuestionCate;
use serde::Serialize;

#[derive(Serialize)]
pub struct QuestionCateResp {
    pub id: i32,
    #[serde(rename(serialize = "relatedId"))]
    pub related_id: i32,
    pub label: String,
    pub key: String,
    #[serde(rename(serialize = "sortOrder"))]
    pub sort_order: i32,
}

impl From<QuestionCate> for QuestionCateResp {
    fn from(row: QuestionCate) -> Self {
        Self {
            id: row.id,
            related_id: row.related_id,
            label: row.label,
            key: row.key,
            sort_order: row.sort_order,
        }
    }
}
