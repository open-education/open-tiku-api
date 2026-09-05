use crate::api::resp::paper::CommonPaperResp;
use crate::enums::test::{TestMethod, TestResult, TestStatus};
use crate::model::homework_student_test_answer::HomeworkStudentTestAnswer;
use crate::model::homework_student_test_attempt::HomeworkStudentTestAttempt;
use crate::util::local::to_local_datetime;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize)]
pub struct InfoResp {
    pub id: i64,
    #[serde(rename(serialize = "homeworkId"))]
    pub homework_id: i64,
    #[serde(rename(serialize = "studentId"))]
    pub student_id: i64,
    pub deadline: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
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

#[derive(Serialize)]
pub struct AnswerInfoResp {
    pub id: i64,
    // 做题记录标识
    #[serde(rename(serialize = "attemptId"))]
    pub attempt_id: i64,
    #[serde(rename(serialize = "questionId"))]
    pub question_id: i64,
    // 用户的最终选择/填写内容
    pub answer: String,
    // 是否正确 0 未作答 1 正确 2 错误
    pub result: i16,
    #[serde(rename(serialize = "resultDesc"))]
    pub result_desc: String,
    // 笔记
    pub note: String,
    // 备注
    pub remark: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

impl From<HomeworkStudentTestAnswer> for AnswerInfoResp {
    fn from(row: HomeworkStudentTestAnswer) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            attempt_id: row.attempt_id,
            question_id: row.question_id,
            answer: row.answer,
            result: row.result,
            result_desc: TestResult::desc(row.result),
            note: row.note,
            remark: row.remark,
            created_at: to_local_datetime(row.created_at.unwrap_or_default()),
            updated_at: to_local_datetime(row.updated_at.unwrap_or_default()),
        }
    }
}

#[derive(Serialize)]
pub struct AttemptInfoResp {
    pub id: i64,
    #[serde(rename(serialize = "studentId"))]
    pub student_id: i64,
    #[serde(rename(serialize = "homeworkId"))]
    pub homework_id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    // 刷题轮次/批次 第1次刷 第2次刷...
    #[serde(rename(serialize = "attemptNumber"))]
    pub attempt_number: i16,
    // 训练方法 1 练习模式 2 考试模式
    pub method: i16,
    #[serde(rename(serialize = "methodDesc"))]
    pub method_desc: String,
    // 状态：1 进行中 2 已交卷
    pub status: i16,
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    // 最终总得分 交卷前为0
    pub score: Decimal,
    // 开始时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    // 进度更新时间, 减去开始时间为耗时
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
    // 交卷时间
    #[serde(rename(serialize = "completedAt"))]
    pub completed_at: String,

    pub answers: Vec<AnswerInfoResp>,
}

impl From<HomeworkStudentTestAttempt> for AttemptInfoResp {
    fn from(row: HomeworkStudentTestAttempt) -> Self {
        Self {
            id: row.id.unwrap_or_default(),
            student_id: row.student_id,
            homework_id: row.homework_id,
            class_id: row.class_id,
            paper_id: row.paper_id,
            attempt_number: row.attempt_number,
            method: row.method,
            method_desc: TestMethod::desc(row.method),
            status: row.status,
            status_desc: TestStatus::desc(row.status),
            score: row.score.unwrap_or_default(),
            created_at: to_local_datetime(row.created_at.unwrap_or_default()),
            updated_at: to_local_datetime(row.updated_at.unwrap_or_default()),
            completed_at: if row.completed_at.is_some() {
                to_local_datetime(row.completed_at.unwrap_or_default())
            } else {
                "".to_string()
            },
            answers: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct AttemptListResp {
    pub list: Vec<AttemptInfoResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}
