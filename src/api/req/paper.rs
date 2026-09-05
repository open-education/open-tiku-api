use crate::model::paper_gen_config::{DifficultyLevelInfo, QuestionTypeInfo};
use crate::model::question::{Content, QuestionOption};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Deserialize)]
pub struct CommonPaperReq {
    pub id: Option<i64>,
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(deserialize = "relatedName"))]
    pub related_name: String,
    #[serde(rename(deserialize = "paperType"))]
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
pub struct TopPaperGroupReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(deserialize = "subTitle"))]
    pub sub_title: Option<String>,
    pub questions: Vec<TopPaperQuestionReq>,
}

#[derive(Deserialize)]
pub struct TopPaperQuestionReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "orderNum"))]
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    #[serde(rename(deserialize = "optionsLayout"))]
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

#[derive(Deserialize)]
pub struct PaperListReq {
    pub source: String,
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(deserialize = "paperType"))]
    pub paper_type: Option<i16>,
    pub tag: Option<String>,
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    pub status: Option<i16>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

// 以下为配置信息
#[derive(Deserialize, Serialize)]
pub struct GenPaperGenConfig {
    #[serde(rename(deserialize = "questionCateIds", serialize = "questionCateIds"))]
    pub question_cate_ids: Vec<i32>,
    #[serde(rename(deserialize = "tagIds", serialize = "tagIds"))]
    pub tag_ids: Option<Vec<i16>>,
    #[serde(rename(deserialize = "dimensionIds", serialize = "dimensionIds"))]
    pub dimension_ids: Option<Vec<i16>>,
    #[serde(rename(deserialize = "levelRange", serialize = "levelRange"))]
    pub level_range: DifficultyLevelInfo,
    #[serde(rename(deserialize = "questionTypes", serialize = "questionTypes"))]
    pub question_types: Vec<QuestionTypeInfo>,
}

#[derive(Deserialize)]
pub struct GenPaperPreviewReq {
    pub common: CommonPaperReq,
    pub conf: GenPaperGenConfig,
}

#[derive(Deserialize)]
pub struct PaperGenQuestionReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "orderNum"))]
    pub order_num: i16,
    #[serde(rename(deserialize = "questionId"))]
    pub question_id: i64,
    pub score: i32,
}

#[derive(Deserialize)]
pub struct PaperGenGroupReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(deserialize = "subTitle"))]
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
