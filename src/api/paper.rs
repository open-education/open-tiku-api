use crate::middleware::user::UserInfo;
use crate::model::question::{Content, QuestionOption};
use crate::service::paper;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

/// 试卷相关操作

#[derive(Deserialize)]
pub struct PaperCommonReq {
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
    pub status: i16,
    pub source: String,
    pub remark: Option<String>,
}

#[derive(Deserialize)]
pub struct PaperTopReq {
    #[serde(flatten)]
    pub common: PaperCommonReq,
    pub groups: Vec<PaperTopGroupReq>,
}

#[derive(Deserialize)]
pub struct PaperTopGroupReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(deserialize = "subTitle"))]
    pub sub_title: Option<String>,
    pub questions: Vec<PaperTopQuestionReq>,
}

#[derive(Deserialize)]
pub struct PaperTopQuestionReq {
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

// 添加精选试卷
#[post("/top/add")]
pub async fn top_add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<PaperTopReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paper::top_add(app_conf, req.into_inner(), user_info).await)
}

// 查看精选试卷详情
#[derive(Serialize)]
pub struct PaperResp {
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
    #[serde(rename(deserialize = "statusDesc"))]
    pub status_desc: String,
    #[serde(rename(serialize = "approveId"))]
    pub approve_id: i64, // 审核人
    #[serde(rename(serialize = "rejectReason"))]
    pub reject_reason: Option<String>, // 拒绝原因
    #[serde(rename(serialize = "approveAt"))]
    pub approve_at: Option<String>, // 审核时间

    pub remark: Option<String>,
    pub count: i32,
    pub groups: Vec<PaperGroupResp>,

    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct PaperGroupResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(serialize = "subTitle"))]
    pub sub_title: Option<String>,
    pub questions: Vec<PaperQuestionResp>,
}

#[derive(Serialize)]
pub struct PaperQuestionResp {
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

#[get("/top/info/{id}")]
pub async fn top_info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i64,)>,
) -> ApiResponse<PaperResp> {
    ApiResponse::response(paper::top_info(app_conf, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct PaperListReq {
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(deserialize = "paperType"))]
    pub paper_type: Option<i16>,
    pub tag: Option<String>,
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct PaperListResp {
    pub list: Vec<PaperResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("/list")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<PaperListReq>,
) -> ApiResponse<PaperListResp> {
    ApiResponse::response(paper::list(app_conf, req.into_inner()).await)
}

#[get("/latest/{count}")]
pub async fn latest(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i64,)>,
) -> ApiResponse<Vec<PaperResp>> {
    ApiResponse::response(paper::latest(app_conf, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct PaperGenLevelRangeReq {
    pub basic: i16,   // 基础题百分比
    pub improve: i16, // 提升题百分比
    pub expand: i16,  // 扩展题百分比
}

#[derive(Deserialize)]
pub struct PaperGenQuestionTypeReq {
    pub id: i16,
    pub label: String,
    pub num: i16,
    pub score: i16,
}

#[derive(Deserialize)]
pub struct PaperGenConfigReq {
    // 题型标识列表
    #[serde(rename(deserialize = "questionCateIds"))]
    pub question_cate_ids: Vec<i32>,
    #[serde(rename(deserialize = "tagIds"))]
    pub tag_ids: Option<Vec<i16>>,
    #[serde(rename(deserialize = "dimensionIds"))]
    pub dimension_ids: Option<Vec<i16>>,
    #[serde(rename(deserialize = "levelRange"))]
    pub level_range: Option<PaperGenLevelRangeReq>,
    #[serde(rename(deserialize = "questionTypes"))]
    pub question_types: Vec<PaperGenQuestionTypeReq>,
}

#[derive(Deserialize)]
pub struct PaperGenReq {
    #[serde(flatten)]
    pub common: PaperCommonReq,
    pub conf: PaperGenConfigReq,
}

#[post("/gen/preview")]
pub async fn preview(
    app_conf: web::Data<AppConfig>,
    req: web::Json<PaperGenReq>,
    user_info: UserInfo,
) -> ApiResponse<PaperResp> {
    ApiResponse::response(paper::preview(app_conf, req.into_inner(), user_info).await)
}
