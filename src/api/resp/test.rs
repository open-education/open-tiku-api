use crate::api::resp::paper::CommonPaperResp;
use crate::enums::test::{TestMethod, TestResult, TestStatus};
use crate::model::test_answer::TestAnswer;
use crate::model::test_attempt::TestAttempt;
use crate::util::local::to_local_datetime;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoResp {
    pub id: i64,
    pub homework_id: i64,
    pub student_id: i64,
    pub deadline: String,
    pub created_at: String,
    pub updated_at: String,
    pub paper_info: CommonPaperResp,
}

// 任务信息列表
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResp {
    pub list: Vec<InfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerInfoResp {
    pub id: i64,
    pub attempt_id: i64,
    pub question_id: i64,
    // 用户的最终选择/填写内容
    pub answer: String,
    // 是否正确 0 未作答 1 正确 2 错误
    pub result: i16,
    pub result_desc: String,
    // 笔记
    pub note: String,
    // 备注
    pub remark: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TestAnswer> for AnswerInfoResp {
    fn from(row: TestAnswer) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            attempt_id: row.attempt_id,
            question_id: row.question_id,
            answer: row.answer,
            result: row.result,
            result_desc: TestResult::desc(row.result).to_string(),
            note: row.note,
            remark: row.remark,
            created_at: to_local_datetime(row.created_at),
            updated_at: to_local_datetime(row.updated_at),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptInfoResp {
    pub id: i64,
    pub student_id: i64,
    pub homework_id: i64,
    pub class_id: i64,
    pub paper_id: i64,
    // 刷题轮次/批次 第1次刷 第2次刷...
    pub attempt_number: i16,
    // 训练方法 1 练习模式 2 考试模式
    pub method: i16,
    pub method_desc: String,
    // 状态：1 进行中 2 已交卷
    pub status: i16,
    pub status_desc: String,
    // 最终总得分 交卷前为0
    pub score: Decimal,
    // 开始时间
    pub created_at: String,
    // 进度更新时间, 减去开始时间为耗时
    pub updated_at: String,
    // 交卷时间
    pub completed_at: String,

    pub answers: Vec<AnswerInfoResp>,
}

impl From<TestAttempt> for AttemptInfoResp {
    fn from(row: TestAttempt) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            student_id: row.student_id,
            homework_id: row.homework_id,
            class_id: row.class_id,
            paper_id: row.paper_id,
            attempt_number: row.attempt_number,
            method: row.method,
            method_desc: TestMethod::desc(row.method).to_string(),
            status: row.status,
            status_desc: TestStatus::desc(row.status).to_string(),
            score: row.score.unwrap_or_default(),
            created_at: to_local_datetime(row.created_at),
            updated_at: to_local_datetime(row.updated_at),
            completed_at: to_local_datetime(row.completed_at),
            answers: vec![],
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptListResp {
    pub list: Vec<AttemptInfoResp>,
    pub page_no: i32,
    pub page_size: i32,
    pub total: i64,
}
