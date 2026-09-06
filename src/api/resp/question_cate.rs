use crate::model::question_cate::QuestionCate;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionCateResp {
    pub id: i32,
    pub related_id: i32,
    pub label: String,
    pub key: String,
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
