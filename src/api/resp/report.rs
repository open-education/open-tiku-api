use crate::model::paper::FileInfo;
use crate::model::report::Report;
use crate::util::local::to_local_datetime;
use serde::Serialize;
use sqlx::types::Json;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportResp {
    pub id: i64,
    pub report_type: i16,
    pub report_id: i64,
    pub value: String,
    pub label: String,
    pub label_desc: String,
    pub content: String,
    pub source_url: String,
    pub file_list: Option<Json<Vec<FileInfo>>>,
    pub status: i16,
    pub approve_id: Option<i64>,
    pub approve_name: String,
    pub reject_reason: Option<String>,
    pub approve_at: String,
    pub remark: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<(Report, String)> for ReportResp {
    fn from(row: (Report, String)) -> Self {
        let raw = row.0;
        Self {
            id: raw.id.unwrap_or(0),
            report_type: raw.report_type,
            report_id: raw.report_id,
            value: raw.value,
            label: raw.label,
            label_desc: raw.label_desc,
            content: raw.content,
            source_url: raw.source_url,
            file_list: raw.file_list,
            status: raw.status,
            approve_id: raw.approve_id,
            approve_name: row.1,
            reject_reason: raw.reject_reason,
            approve_at: to_local_datetime(raw.approve_at),
            remark: raw.remark,
            created_at: to_local_datetime(Some(raw.created_at)),
            updated_at: to_local_datetime(Some(raw.updated_at)),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportListResp {
    pub list: Vec<ReportResp>,
    pub page_no: i64,
    pub page_size: i64,
    pub total: i64,
}
