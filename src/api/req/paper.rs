use crate::model::paper::FileInfo;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperAddReq {
    pub id: Option<i64>,
    pub two_level_id: i32,
    pub five_level_id: Option<i32>,
    pub paper_type: i16,
    pub file_type: i16,
    pub file_list: Option<Vec<FileInfo>>,
    pub province_id: Option<i32>,
    pub city_id: Option<i32>,
    pub school_name: Option<String>,
    pub exam_type: i16,
    pub year: String,
    pub title: String,
    pub score: Option<i32>,
    pub duration: Option<i32>,
    pub level_type: i16,
    pub difficulty_type: i16,
    pub source: Option<String>,
    pub remark: String,
    pub count: Option<i32>,
    pub status: i16,
}

// 试卷搜索
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperListReq {
    pub source: String,
    pub two_level_id: i32,
    pub five_level_id: Option<i32>,
    pub year: Option<String>,
    pub exam_type: Option<i16>,
    pub level_type: Option<i16>,
    pub difficulty_type: Option<i16>,
    pub province_id: Option<i32>,
    pub city_id: Option<i32>,
    pub school_name: Option<String>,
    pub status: Option<i16>,
    pub page_no: i64,
    pub page_size: i64,
}

#[derive(Deserialize)]
pub struct DeleteReq {
    pub id: i64,
}
