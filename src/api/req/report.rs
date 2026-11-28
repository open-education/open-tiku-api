use crate::model::paper::FileInfo;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddReq {
    pub id: Option<i64>,
    pub report_type: i16,
    pub report_id: i64,
    pub value: String,
    pub label: String,
    pub label_desc: String,
    pub content: String,
    pub source_url: String,
    pub file_list: Option<Vec<FileInfo>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListReq {
    pub status: Option<i16>,
    pub page_no: i64,
    pub page_size: i64,
}
