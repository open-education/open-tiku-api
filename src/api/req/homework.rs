use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeworkAddReq {
    pub batch_no: i32,
    pub paper_id: i64,
    pub title: String,
    pub deadline: String,
    pub remark: Option<String>,
    pub class_map: HashMap<i64, Vec<i64>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeworkListReq {
    pub paper_id: i64,
    pub batch_no: Option<i32>,
    pub page_no: i32,
    pub page_size: i32,
}
