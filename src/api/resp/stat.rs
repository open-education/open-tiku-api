use crate::model::question::LightQuestion;
use crate::util::local::{to_local_datetime, to_time_ago};
use serde::Serialize;

// 首页返回
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResp {
    // 最新上传题目
    pub latest_questions: Vec<LatestQuestionResp>,
}

// 最新题目只展示基本信息
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestQuestionResp {
    pub id: i64,
    pub question_cate_id: i32,
    pub title: String,
    pub created_at: String,
    pub time_desc: String,
}

impl From<LightQuestion> for LatestQuestionResp {
    fn from(raw: LightQuestion) -> Self {
        Self {
            id: raw.id,
            question_cate_id: raw.question_cate_id,
            title: raw.title,
            created_at: to_local_datetime(Some(raw.created_at)),
            time_desc: to_time_ago(Some(raw.created_at)),
        }
    }
}
