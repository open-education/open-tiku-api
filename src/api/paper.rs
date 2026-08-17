use crate::api::question::QuestionInfoResp;
use crate::app::config::AppState;
use crate::middleware::user::UserInfo;
use crate::model::paper_gen_config::{DifficultyLevelInfo, QuestionTypeInfo};
use crate::model::question::{Content, QuestionOption};
use crate::service::paper;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

/// 试卷相关操作

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

// 添加精选试卷
#[post("/top/add")]
pub async fn top_add(
    app_state: web::Data<AppState>,
    req: web::Json<TopPaperReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paper::top_add(app_state, req.into_inner(), user_info).await)
}

// 查看精选试卷详情
#[derive(Serialize)]
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

#[get("/top/info/{id}")]
pub async fn top_info(
    app_state: web::Data<AppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<TopPaperResp> {
    ApiResponse::response(paper::top_info(app_state, path.into_inner().0).await)
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

#[derive(Serialize)]
pub struct PaperListResp {
    pub list: Vec<CommonPaperResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<PaperListReq>,
    user_info: Option<UserInfo>,
) -> ApiResponse<PaperListResp> {
    ApiResponse::response(paper::list(app_state, req.into_inner(), user_info).await)
}

#[get("/latest/{paperType}/{count}")]
pub async fn latest(
    app_state: web::Data<AppState>,
    path: web::Path<(i16, i64)>,
) -> ApiResponse<Vec<CommonPaperResp>> {
    ApiResponse::response(paper::latest(app_state, path.into_inner()).await)
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

#[derive(Serialize)]
pub struct GenPaperQuestionResp {
    pub common: CommonPaperGenQuestionResp,
    pub info: QuestionInfoResp,
}

#[post("/gen/preview")]
pub async fn preview(
    app_state: web::Data<AppState>,
    req: web::Json<GenPaperPreviewReq>,
    user_info: UserInfo,
) -> ApiResponse<GenPaperResp> {
    ApiResponse::response(paper::preview(app_state, req.into_inner(), user_info).await)
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

// 保存试卷
#[post("/gen/add")]
pub async fn gen_add(
    app_state: web::Data<AppState>,
    req: web::Json<PaperGenReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paper::gen_add(app_state, req.into_inner(), user_info).await)
}

#[get("/gen/info/{id}")]
pub async fn gen_info(
    app_state: web::Data<AppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<GenPaperResp> {
    ApiResponse::response(paper::gen_info(app_state, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct DeleteReq {
    pub id: i64,
}

#[post("/delete")]
pub async fn delete(
    app_state: web::Data<AppState>,
    req: web::Json<DeleteReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(paper::delete(app_state, req.into_inner(), user_info).await)
}
