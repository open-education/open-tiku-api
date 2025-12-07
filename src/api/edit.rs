/// 编辑
/// 目前涉及到索引文件的几个选项 问题类型, 标签, 评分和图片在同一章节下面同时更新存在并发问题最后更新的会最终写入文件暂时不处理
/// 其它片段是单独的文件不存在冲突, 仅涉及多个人同时修改同一片段内容时存在覆盖问题
/// 后续将索引文件细化后能有效避免这类问题
///
use crate::service::edit;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EditQuestionTypeReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "questionType"))]
    pub question_type: String,
}

#[post("/question-type")]
pub async fn edit_question_type(req: web::Json<EditQuestionTypeReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_question_type(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditTagsReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "tags"))]
    pub tags: Vec<String>,
}

#[post("/tags")]
pub async fn edit_tags(req: web::Json<EditTagsReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_tags(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditRateReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "rate"))]
    pub rate: String,
}

#[post("/rate")]
pub async fn edit_rate(req: web::Json<EditRateReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_rate(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditTitleReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "title"))]
    pub title: String,
}

#[post("/title")]
pub async fn edit_title(req: web::Json<EditTitleReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_title(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditMentionReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "mention"))]
    pub mention: String,
}

#[post("/mention")]
pub async fn edit_mention(req: web::Json<EditMentionReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_mention(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "a"))]
    pub a: String,
}

#[post("/a")]
pub async fn edit_a(req: web::Json<EditAReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_a(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditBReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "b"))]
    pub b: String,
}

#[post("/b")]
pub async fn edit_b(req: web::Json<EditBReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_b(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditCReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "c"))]
    pub c: String,
}

#[post("/c")]
pub async fn edit_c(req: web::Json<EditCReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_c(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditDReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "d"))]
    pub d: String,
}

#[post("/d")]
pub async fn edit_d(req: web::Json<EditDReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_d(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditEReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "e"))]
    pub e: String,
}

#[post("/e")]
pub async fn edit_e(req: web::Json<EditEReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_e(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAnswerReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "answer"))]
    pub answer: String,
}

#[post("/answer")]
pub async fn edit_answer(req: web::Json<EditAnswerReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_answer(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditKnowledgeReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "knowledge"))]
    pub knowledge: String,
}

#[post("/knowledge")]
pub async fn edit_knowledge(req: web::Json<EditKnowledgeReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_knowledge(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditAnalyzeReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "analyze"))]
    pub analyze: String,
}

#[post("/analyze")]
pub async fn edit_analyze(req: web::Json<EditAnalyzeReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_analyze(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditProcessReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "process"))]
    pub process: String,
}

#[post("/process")]
pub async fn edit_process(req: web::Json<EditProcessReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_process(req.into_inner()))
}

#[derive(Deserialize)]
pub struct EditRemarkReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "remark"))]
    pub remark: String,
}

#[post("/remark")]
pub async fn edit_remark(req: web::Json<EditRemarkReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_remark(req.into_inner()))
}
