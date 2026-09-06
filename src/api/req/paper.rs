use crate::model::paper_gen_config::{DifficultyLevelInfo, QuestionTypeInfo};
use crate::model::question::{Content, QuestionOption};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonPaperReq {
    pub id: Option<i64>,
    pub related_id: i32,
    pub related_name: String,
    pub paper_type: i16,
    pub tag: String,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub title: String,
    pub score: i32,
    pub count: Option<i32>,
    pub status: i16,
    pub source: String,
    pub remark: Option<String>,
}

#[derive(Deserialize)]
pub struct TopPaperReq {
    pub common: CommonPaperReq,
    pub groups: Vec<TopPaperGroupReq>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopPaperGroupReq {
    pub gen_id: String,
    pub type_name: String,
    pub sub_title: Option<String>,
    pub questions: Vec<TopPaperQuestionReq>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopPaperQuestionReq {
    pub gen_id: String,
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperListReq {
    pub source: String,
    pub related_id: i32,
    pub paper_type: Option<i16>,
    pub tag: Option<String>,
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    pub status: Option<i16>,
    pub page_no: i32,
    pub page_size: i32,
}

// 以下为配置信息
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenPaperGenConfig {
    pub question_cate_ids: Vec<i32>,
    pub tag_ids: Option<Vec<i16>>,
    pub dimension_ids: Option<Vec<i16>>,
    pub level_range: DifficultyLevelInfo,
    pub question_types: Vec<QuestionTypeInfo>,
}

#[derive(Deserialize)]
pub struct GenPaperPreviewReq {
    pub common: CommonPaperReq,
    pub conf: GenPaperGenConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperGenQuestionReq {
    pub gen_id: String,
    pub order_num: i16,
    pub question_id: i64,
    pub score: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperGenGroupReq {
    pub gen_id: String,
    pub type_name: String,
    pub sub_title: Option<String>,
    pub questions: Vec<PaperGenQuestionReq>,
}

#[derive(Deserialize)]
pub struct PaperGenReq {
    pub common: CommonPaperReq,
    pub conf: GenPaperGenConfig,
    pub groups: Vec<PaperGenGroupReq>,
}

#[derive(Deserialize)]
pub struct DeleteReq {
    pub id: i64,
}
