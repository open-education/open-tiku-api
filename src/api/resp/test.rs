use crate::api::resp::paper::CommonPaperResp;
use serde::Serialize;

#[derive(Serialize)]
pub struct InfoResp {
    pub id: i64,
    pub deadline: String,
    #[serde(rename(serialize = "paperInfo"))]
    pub paper_info: CommonPaperResp,
}

// 任务信息列表
#[derive(Serialize)]
pub struct ListResp {
    pub list: Vec<InfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
