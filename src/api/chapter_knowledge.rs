use crate::api::textbook::TextbookResp;
use crate::service::chapter_knowledge;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateChapterKnowledgeReq {
    #[serde(rename(deserialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

#[derive(Debug, Serialize)]
pub struct ChapterKnowledgeResp {
    pub id: Option<i32>,
    #[serde(rename(serialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateChapterKnowledgeReq>,
) -> ApiResponse<ChapterKnowledgeResp> {
    info!("req: {:?}", req);
    ApiResponse::response(chapter_knowledge::add(app_conf, req.into_inner()).await)
}

#[get("/info/{chapter_or_knowledge_id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<ChapterKnowledgeResp> {
    info!("path: {:?}", path);
    ApiResponse::response(
        chapter_knowledge::info_by_chapter_or_knowledge(app_conf, path.into_inner().0).await,
    )
}

#[derive(Debug, Deserialize)]
pub struct ChapterKnowledgeIdsReq {
    pub ids: Vec<i32>,
}

#[post("/knowledge")]
pub async fn knowledge(
    app_conf: web::Data<AppConfig>,
    req: web::Json<ChapterKnowledgeIdsReq>,
) -> ApiResponse<Vec<TextbookResp>> {
    info!("req: {:?}", req);
    ApiResponse::response(chapter_knowledge::get_by_chapter(app_conf, req.into_inner()).await)
}

#[post("/chapter")]
pub async fn chapter(
    app_conf: web::Data<AppConfig>,
    req: web::Json<ChapterKnowledgeIdsReq>,
) -> ApiResponse<Vec<TextbookResp>> {
    info!("req: {:?}", req);
    ApiResponse::response(chapter_knowledge::get_by_knowledge(app_conf, req.into_inner()).await)
}

#[derive(Debug, Deserialize)]
pub struct RemoveChapterKnowledgeReq {
    pub id: i32,
}

#[post("/remove")]
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: web::Json<RemoveChapterKnowledgeReq>,
) -> ApiResponse<bool> {
    info!("req: {:?}", req);
    ApiResponse::response(chapter_knowledge::remove(app_conf, req.into_inner()).await)
}
