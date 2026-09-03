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

#[derive(Deserialize)]
pub struct AnswerAddReq {
    #[serde(rename(deserialize = "questionId"))]
    pub question_id: i64,
    // 用户的最终选择/填写内容
    pub answer: String,
    // 是否正确 0 未作答 1 正确 2 错误
    pub result: i16,
    // 笔记
    pub note: String,
}

#[derive(Deserialize)]
pub struct TestAnswerAddReq {
    #[serde(rename(deserialize = "attemptId"))]
    pub attempt_id: i64,
    pub status: i16,
    pub list: Vec<AnswerAddReq>,
}
