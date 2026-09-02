use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListReq {
    #[serde(rename(deserialize = "startDate"))]
    pub start_date: String,
    #[serde(rename(deserialize = "endDate"))]
    pub end_date: String,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Deserialize)]
pub struct LatestAttemptReq {
    pub id: i64,
    pub method: i16,
}

// 做题记录, 进行中的一直复用, 交卷后重新生成下一次做题记录
#[derive(Deserialize)]
pub struct TestAttemptReq {
    pub id: Option<i64>,
    #[serde(rename(deserialize = "homeworkId"))]
    pub homework_id: i64,
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(deserialize = "paperId"))]
    pub paper_id: i64,
    // 刷题轮次/批次 第1次刷 第2次刷...
    #[serde(rename(deserialize = "attemptNumber"))]
    pub attempt_number: i16,
    // 训练方法 1 练习模式 2 考试模式
    pub method: i16,
    // 状态：1 进行中 2 已交卷
    pub status: i16,
    // 进度更新时间, 减去开始时间为耗时
    pub updated_at: Option<DateTime<Utc>>,
    // 交卷时间
    pub completed_at: Option<DateTime<Utc>>,
}
