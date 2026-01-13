use crate::model::question::{Content, QuestionOption};
use crate::service::edit;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{post, web};
use rust_decimal::Decimal;
use serde::Deserialize;

/// 编辑
#[derive(Deserialize)]
pub struct EditQuestionTypeReq {
    pub id: i64,
    #[serde(rename(deserialize = "questionType"))]
    pub question_type: i32,
}

// 题目类型
#[post("/question-type")]
pub async fn question_type(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditQuestionTypeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::question_type(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditTagsReq {
    pub id: i64,
    pub tags: Vec<i32>,
}

// 题目标签
#[post("/tags")]
pub async fn tags(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTagsReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::tags(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditRateReq {
    pub id: i64,
    // 使用 rust_decimal 处理 0.5 精度问题
    #[serde(rename(deserialize = "difficultyLevel"))]
    pub difficulty_level: Decimal, // 题目难易程度
}

// 题目难易程度
#[post("/rate")]
pub async fn rate(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRateReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::rate(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditTitleReq {
    pub id: i64,
    pub title: String,
}

// 题目标题
#[post("/title")]
pub async fn title(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTitleReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::title(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditMentionReq {
    pub id: i64,
    pub mention: String,
}

// 标题补充说明
#[post("/mention")]
pub async fn mention(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditMentionReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::mention(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditSelectLayoutReq {
    pub id: i64,
    pub layout: i16,
}

// 选项样式
#[post("/options-layout")]
pub async fn options_layout(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditSelectLayoutReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::options_layout(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditSelectReq {
    pub id: i64,
    pub option: QuestionOption,
}

// 编辑选项
#[post("/options")]
pub async fn options(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditSelectReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::options(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditAnswerReq {
    pub id: i64,
    pub answer: String,
}

// 编辑答案
#[post("/answer")]
pub async fn answer(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnswerReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::answer(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditKnowledgeReq {
    pub id: i64,
    pub knowledge: String,
}

// 编辑知识点
#[post("/knowledge")]
pub async fn knowledge(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditKnowledgeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::knowledge(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditAnalyzeReq {
    pub id: i64,
    pub analyze: Content,
}

// 解题分析
#[post("/analyze")]
pub async fn analyze(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnalyzeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::analyze(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditProcessReq {
    pub id: i64,
    pub process: Content,
}

// 解题过程
#[post("/process")]
pub async fn process(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditProcessReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::process(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditRemarkReq {
    pub id: i64,
    pub remark: String,
}

#[post("/remark")]
pub async fn remark(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRemarkReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::remark(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditStatusReq {
    pub id: i64,
    pub status: i16,
    #[serde(rename(deserialize = "rejectReason"))]
    pub reject_reason: Option<String>,
}

// 更新状态
#[post("/status")]
pub async fn status(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditStatusReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::status(app_conf, req.into_inner()).await)
}
