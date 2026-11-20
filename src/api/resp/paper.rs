use crate::model::paper::{CountPaper, FileInfo, Paper};
use crate::util::local::to_local_datetime;
use serde::Serialize;
use sqlx::types::Json;

#[derive(Serialize)]
pub struct CountPaperResp {
    pub status: i16,
    pub count: i64,
}

impl From<CountPaper> for CountPaperResp {
    fn from(raw: CountPaper) -> Self {
        Self {
            status: raw.status,
            count: raw.count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperResp {
    pub id: i64,
    pub two_level_id: i32,
    pub five_level_id: Option<i32>,
    pub paper_type: i16,
    pub file_type: i16,
    pub file_list: Option<Json<Vec<FileInfo>>>,
    pub province_id: Option<i32>,
    pub city_id: Option<i32>,
    pub school_name: Option<String>,
    pub exam_type: i16,
    pub year: String,
    pub title: String,
    pub score: Option<i32>,
    pub duration: Option<i16>,
    pub level_type: i16,
    pub difficulty_type: i16,
    pub source: Option<String>,
    pub remark: String,
    pub count: Option<i32>,
    pub status: i16,
    pub author_id: i64,
    pub author_name: String,
    pub approve_id: Option<i64>,
    pub approve_name: String,
    pub reject_reason: Option<String>,
    pub approve_at: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<(Paper, String, String)> for PaperResp {
    fn from(row: (Paper, String, String)) -> Self {
        let raw = row.0;
        Self {
            id: raw.id.unwrap_or_default(),
            two_level_id: raw.two_level_id,
            five_level_id: raw.five_level_id,
            paper_type: raw.paper_type,
            file_type: raw.file_type,
            file_list: raw.file_list,
            province_id: raw.province_id,
            city_id: raw.city_id,
            school_name: raw.school_name,
            exam_type: raw.exam_type,
            year: raw.year,
            title: raw.title,
            score: raw.score,
            duration: raw.duration,
            level_type: raw.level_type,
            difficulty_type: raw.difficulty_type,
            source: raw.source,
            remark: raw.remark,
            count: raw.count,
            status: raw.status,
            author_id: raw.author_id,
            author_name: row.1,
            approve_id: raw.approve_id,
            approve_name: row.2,
            reject_reason: raw.reject_reason,
            approve_at: to_local_datetime(raw.approve_at),
            created_at: to_local_datetime(Some(raw.created_at)),
            updated_at: to_local_datetime(Some(raw.updated_at)),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperListResp {
    pub list: Vec<PaperResp>,
    pub page_no: i64,
    pub page_size: i64,
    pub total: i64,
}
