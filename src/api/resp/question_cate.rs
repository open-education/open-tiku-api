use crate::api::resp::textbook::TextbookResp;
use crate::model::question_cate::QuestionCate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
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

// 题型父级菜单信息
#[derive(Serialize, Deserialize)]
pub struct QuestionCateListResp {
    pub info_map: HashMap<i32, QuestionCateResp>,
    pub parent_map: HashMap<i32, TextbookResp>,
}
