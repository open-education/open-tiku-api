use crate::service::edit;
use crate::util::response::ApiResponse;
/// 编辑
use crate::AppConfig;
use actix_web::{post, web};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EditQuestionTypeReq {
    pub id: i64,

    #[serde(rename(deserialize = "questionType"))]
    pub question_type: i32,
}

#[post("/question-type")]
pub async fn edit_question_type(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditQuestionTypeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_question_type(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditTagsReq {
    pub id: i64,

    #[serde(rename(deserialize = "tags"))]
    pub tags: Vec<i32>,
}

#[post("/tags")]
pub async fn edit_tags(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTagsReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_tags(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditRateReq {
    pub id: i64,

    // 使用 rust_decimal 处理 0.5 精度问题
    #[serde(rename(deserialize = "difficultyLevel"))]
    pub difficulty_level: Decimal, // 题目难易程度
}

#[post("/rate")]
pub async fn edit_rate(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRateReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_rate(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditTitleReq {
    pub id: i64,

    #[serde(rename(deserialize = "title"))]
    pub title: String,
}

#[post("/title")]
pub async fn edit_title(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTitleReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_title(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct EditSelectReq {
    pub id: String,

    #[serde(rename(deserialize = "select"))]
    pub select: String,
}

#[post("/select")]
pub async fn edit_select(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditSelectReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_select(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditMentionReq {
    pub id: String,

    #[serde(rename(deserialize = "mention"))]
    pub mention: String,
}

#[post("/mention")]
pub async fn edit_mention(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditMentionReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_mention(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAReq {
    pub id: String,

    #[serde(rename(deserialize = "a"))]
    pub a: String,
}

#[post("/a")]
pub async fn edit_a(app_conf: web::Data<AppConfig>, req: web::Json<EditAReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_a(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditBReq {
    pub id: String,

    #[serde(rename(deserialize = "b"))]
    pub b: String,
}

#[post("/b")]
pub async fn edit_b(app_conf: web::Data<AppConfig>, req: web::Json<EditBReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_b(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditCReq {
    pub id: String,

    #[serde(rename(deserialize = "c"))]
    pub c: String,
}

#[post("/c")]
pub async fn edit_c(app_conf: web::Data<AppConfig>, req: web::Json<EditCReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_c(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditDReq {
    pub id: String,

    #[serde(rename(deserialize = "d"))]
    pub d: String,
}

#[post("/d")]
pub async fn edit_d(app_conf: web::Data<AppConfig>, req: web::Json<EditDReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_d(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditEReq {
    pub id: String,

    #[serde(rename(deserialize = "e"))]
    pub e: String,
}

#[post("/e")]
pub async fn edit_e(app_conf: web::Data<AppConfig>, req: web::Json<EditEReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_e(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAnswerReq {
    pub id: String,

    #[serde(rename(deserialize = "answer"))]
    pub answer: String,
}

#[post("/answer")]
pub async fn edit_answer(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnswerReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_answer(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditKnowledgeReq {
    pub id: String,

    #[serde(rename(deserialize = "knowledge"))]
    pub knowledge: String,
}

#[post("/knowledge")]
pub async fn edit_knowledge(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditKnowledgeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_knowledge(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAnalyzeReq {
    pub id: String,

    #[serde(rename(deserialize = "analyze"))]
    pub analyze: String,
}

#[post("/analyze")]
pub async fn edit_analyze(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnalyzeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_analyze(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditProcessReq {
    pub id: String,

    #[serde(rename(deserialize = "process"))]
    pub process: String,
}

#[post("/process")]
pub async fn edit_process(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditProcessReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_process(app_conf, req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditRemarkReq {
    pub id: String,

    #[serde(rename(deserialize = "remark"))]
    pub remark: String,
}

#[post("/remark")]
pub async fn edit_remark(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRemarkReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_remark(app_conf, req.into_inner()))
}
