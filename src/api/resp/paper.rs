use crate::api::req::paper::GenPaperGenConfig;
use crate::api::resp::question::QuestionInfoResp;
use crate::enums::paper::PaperStatus;
use crate::model::paper::Paper;
use crate::model::paper_gen_question::PaperGenQuestion;
use crate::model::paper_group::PaperGroup;
use crate::model::paper_question::PaperQuestion;
use crate::model::question::{Content, QuestionOption};
use crate::util::local::to_local_datetime;
use serde::Serialize;
use sqlx::types::Json;

#[derive(Serialize, Default)]
pub struct CommonPaperResp {
    pub id: Option<i64>,
    #[serde(rename(serialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(serialize = "relatedName"))]
    pub related_name: String,
    #[serde(rename(serialize = "paperType"))]
    pub paper_type: i16,
    pub tag: String,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub title: String,
    pub score: i32,
    pub source: String,

    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64,
    #[serde(rename(serialize = "authorName"))]
    pub author_name: String,

    // 审核相关
    pub status: i16, // 审核状态
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    #[serde(rename(serialize = "approveId"))]
    pub approve_id: i64, // 审核人
    #[serde(rename(serialize = "rejectReason"))]
    pub reject_reason: Option<String>, // 拒绝原因
    #[serde(rename(serialize = "approveAt"))]
    pub approve_at: Option<String>, // 审核时间

    pub remark: Option<String>,
    pub count: i32,

    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

impl From<Paper> for CommonPaperResp {
    fn from(row: Paper) -> Self {
        Self {
            id: row.id,
            related_id: row.related_id,
            related_name: row.related_name,
            paper_type: row.paper_type,
            tag: row.tag,
            year: row.year,
            grade: row.grade,
            semester: row.semester,
            title: row.title,
            score: row.score,
            source: row.source,
            author_id: row.author_id,
            author_name: row.author_name,
            status: row.status,
            status_desc: PaperStatus::desc(row.status).to_string(),
            approve_id: row.approve_id,
            reject_reason: row.reject_reason,
            approve_at: None,
            remark: row.remark,
            count: row.count,
            created_at: to_local_datetime(Some(row.created_at)),
            updated_at: to_local_datetime(Some(row.updated_at)),
        }
    }
}

#[derive(Serialize)]
pub struct TopPaperResp {
    pub common: CommonPaperResp,
    pub groups: Vec<TopPaperGroupResp>,
}

#[derive(Serialize)]
pub struct CommonPaperGroupResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(serialize = "subTitle"))]
    pub sub_title: Option<String>,
}

impl From<PaperGroup> for CommonPaperGroupResp {
    fn from(row: PaperGroup) -> Self {
        Self {
            id: row.id,
            paper_id: row.paper_id,
            gen_id: row.gen_id,
            type_name: row.type_name,
            sub_title: row.sub_title,
        }
    }
}

#[derive(Serialize)]
pub struct TopPaperGroupResp {
    pub common: CommonPaperGroupResp,
    pub questions: Vec<TopPaperQuestionResp>,
}

#[derive(Serialize)]
pub struct TopPaperQuestionResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "groupId"))]
    pub group_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "orderNum"))]
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    #[serde(rename(serialize = "optionsLayout"))]
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

impl From<PaperQuestion> for TopPaperQuestionResp {
    fn from(row: PaperQuestion) -> Self {
        Self {
            id: row.id,
            paper_id: row.paper_id,
            group_id: row.group_id,
            gen_id: row.gen_id,
            order_num: row.order_num,
            stem: row.stem,
            images: row.images,
            options: row.options,
            options_layout: row.options_layout,
            answer: row.answer,
            analysis: row.analysis,
            score: row.score,
        }
    }
}

#[derive(Serialize)]
pub struct PaperListResp {
    pub list: Vec<CommonPaperResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[derive(Serialize)]
pub struct GenPaperResp {
    pub common: CommonPaperResp,
    pub conf: GenPaperGenConfig,
    pub groups: Vec<GenPaperGroupResp>,
}

#[derive(Serialize)]
pub struct GenPaperGroupResp {
    pub common: CommonPaperGroupResp,
    pub questions: Vec<GenPaperQuestionResp>,
}

#[derive(Serialize)]
pub struct CommonPaperGenQuestionResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "groupId"))]
    pub group_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "orderNum"))]
    pub order_num: i16,
    #[serde(rename(serialize = "questionId"))]
    pub question_id: i64,
    pub score: i32,
}

impl From<PaperGenQuestion> for CommonPaperGenQuestionResp {
    fn from(row: PaperGenQuestion) -> Self {
        Self {
            id: row.id,
            paper_id: row.paper_id,
            group_id: row.group_id,
            gen_id: row.gen_id,
            order_num: row.order_num,
            question_id: row.question_id,
            score: row.score,
        }
    }
}

#[derive(Serialize)]
pub struct GenPaperQuestionResp {
    pub common: CommonPaperGenQuestionResp,
    pub info: QuestionInfoResp,
}
