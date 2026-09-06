use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListReq {
    pub start_date: String,
    pub end_date: String,
    pub page_no: i32,
    pub page_size: i32,
}

#[derive(Deserialize)]
pub struct LatestAttemptReq {
    pub id: i64,
    pub method: i16,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerAddReq {
    pub question_id: i64,
    // 用户的最终选择/填写内容
    pub answer: String,
    // 是否正确 0 未作答 1 正确 2 错误
    pub result: i16,
    // 笔记
    pub note: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestAnswerAddReq {
    pub attempt_id: i64,
    pub status: i16,
    pub list: Vec<AnswerAddReq>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptListReq {
    pub id: i64,
    pub page_no: i32,
    pub page_size: i32,
}
