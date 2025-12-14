use crate::AppConfig;
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
pub async fn edit_question_type(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditQuestionTypeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_question_type(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_tags(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTagsReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_tags(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_rate(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRateReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_rate(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_title(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditTitleReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_title(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
}

#[derive(Deserialize)]
pub struct EditSelectReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,

    #[serde(rename(deserialize = "select"))]
    pub select: String,
}

#[post("/select")]
pub async fn edit_select(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditSelectReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_select(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_mention(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditMentionReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_mention(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_a(app_conf: web::Data<AppConfig>, req: web::Json<EditAReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_a(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_b(app_conf: web::Data<AppConfig>, req: web::Json<EditBReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_b(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_c(app_conf: web::Data<AppConfig>, req: web::Json<EditCReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_c(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_d(app_conf: web::Data<AppConfig>, req: web::Json<EditDReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_d(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_e(app_conf: web::Data<AppConfig>, req: web::Json<EditEReq>) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_e(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_answer(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnswerReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_answer(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_knowledge(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditKnowledgeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_knowledge(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_analyze(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditAnalyzeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_analyze(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_process(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditProcessReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_process(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
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
pub async fn edit_remark(
    app_conf: web::Data<AppConfig>,
    req: web::Json<EditRemarkReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(edit::edit_remark(
        app_conf.meta_path.to_str().unwrap_or(""),
        req.into_inner(),
    ))
}
