use crate::model::question::{AuthorQuestion, LightQuestion, TopTextbook};
use crate::model::stat::CountInfo;
use crate::util::local::{to_local_datetime, to_time_ago};
use serde::Serialize;

// 首页返回
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResp {
    // 网站统计
    pub count_info: CountInfoResp,
    // 最新上传题目
    pub latest_questions: Vec<LatestQuestionResp>,
    // 传题最多的作者
    pub top_teacher_questions: Vec<AuthorQuestionResp>,
    // 题目最多的教材
    pub top_textbooks: Vec<TopTextbookResp>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountInfoResp {
    pub textbook_num: i64,
    pub question_num: i64,
    pub paper_num: i64,
    pub teacher_num: i64,
    pub student_num: i64,
}

impl From<CountInfo> for CountInfoResp {
    fn from(raw: CountInfo) -> Self {
        Self {
            textbook_num: raw.textbook_num,
            question_num: raw.question_num,
            paper_num: raw.paper_num,
            teacher_num: raw.teacher_num,
            student_num: raw.student_num,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorQuestionResp {
    pub author_id: i64,
    pub author_name: String,
    pub cnt: i64,
}

impl From<AuthorQuestion> for AuthorQuestionResp {
    fn from(raw: AuthorQuestion) -> Self {
        Self {
            author_id: raw.author_id,
            author_name: raw.author_name,
            cnt: raw.cnt,
        }
    }
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopTextbookResp {
    pub textbook_id: i32,
    pub cnt: i64,
}

impl From<TopTextbook> for TopTextbookResp {
    fn from(raw: TopTextbook) -> Self {
        Self {
            textbook_id: raw.textbook_id,
            cnt: raw.cnt,
        }
    }
}
