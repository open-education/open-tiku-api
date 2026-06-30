use crate::AppConfig;
use crate::service::chapter_knowledge;
use crate::util::response::ApiResponse;
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
    #[serde(rename(serialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

// 关联章节小节和知识点小类
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateChapterKnowledgeReq>,
) -> ApiResponse<i32> {
    ApiResponse::response(chapter_knowledge::add(app_conf, req.into_inner()).await)
}

// 通过菜单标识获取关联详情-章节小节或者知识点小类标识
#[get("/list/{chapter_or_knowledge_id}")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i32,)>,
) -> ApiResponse<Vec<ChapterKnowledgeResp>> {
    ApiResponse::response(chapter_knowledge::list(app_conf, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct RemoveChapterKnowledgeReq {
    #[serde(rename(deserialize = "chapterId"))]
    pub chapter_id: i32,
    #[serde(rename(deserialize = "knowledgeId"))]
    pub knowledge_id: i32,
}

// 解除绑定关系
#[post("/remove")]
pub async fn remove(
    app_conf: web::Data<AppConfig>,
    req: web::Json<RemoveChapterKnowledgeReq>,
) -> ApiResponse<bool> {
    ApiResponse::response(chapter_knowledge::remove(app_conf, req.into_inner()).await)
}
