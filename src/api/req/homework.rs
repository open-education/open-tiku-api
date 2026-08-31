use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct HomeworkAddReq {
    #[serde(rename(deserialize = "batchNo"))]
    pub batch_no: i32,
    #[serde(rename(deserialize = "paperId"))]
    pub paper_id: i64,
    pub title: String,
    pub remark: Option<String>,
    #[serde(rename(deserialize = "classMap"))]
    pub class_map: HashMap<i64, Vec<i64>>,
}

#[derive(Deserialize)]
pub struct HomeworkListReq {
    #[serde(rename(deserialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(deserialize = "batchNo"))]
    pub batch_no: Option<i32>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}
