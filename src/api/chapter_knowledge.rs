use crate::api::textbook::TextbookResp;
use crate::service::chapter_knowledge;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateChapterKnowledgeReq {
    #[serde(rename(deserialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

#[derive(Serialize)]
pub struct ChapterKnowledgeResp {
    pub id: Option<i32>,
    #[serde(rename(serialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

// 关联章节小节和知识点小类
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateChapterKnowledgeReq>,
) -> ApiResponse<ChapterKnowledgeResp> {
    ApiResponse::response(chapter_knowledge::add(app_conf, req.into_inner()).await)
}

// 通过菜单标识获取关联详情-章节小节或者知识点小类标识
#[get("/info/{chapter_or_knowledge_id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<ChapterKnowledgeResp> {
    ApiResponse::response(
        chapter_knowledge::info_by_chapter_or_knowledge(app_conf, path.into_inner().0).await,
    )
}

#[derive(Deserialize)]
pub struct ChapterKnowledgeIdsReq {
    pub ids: Vec<i32>,
}

// 通过章节小节获取绑定的知识点信息
#[post("/knowledge")]
pub async fn knowledge(
    app_conf: web::Data<AppConfig>,
    req: web::Json<ChapterKnowledgeIdsReq>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(chapter_knowledge::get_by_chapter(app_conf, req.into_inner()).await)
}

// 通过知识点小类标识获取绑定的章节小节信息
#[post("/chapter")]
pub async fn chapter(
    app_conf: web::Data<AppConfig>,
    req: web::Json<ChapterKnowledgeIdsReq>,
) -> ApiResponse<Vec<TextbookResp>> {
    ApiResponse::response(chapter_knowledge::get_by_knowledge(app_conf, req.into_inner()).await)
}

#[derive(Deserialize)]
pub struct RemoveChapterKnowledgeReq {
    pub id: i32,
}

// 解除绑定关系
#[post("/remove")]
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: web::Json<RemoveChapterKnowledgeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(chapter_knowledge::remove(app_conf, req.into_inner()).await)
}
