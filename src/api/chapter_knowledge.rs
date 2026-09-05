use crate::api::req::chapter_knowledge::{CreateChapterKnowledgeReq, RemoveChapterKnowledgeReq};
use crate::api::resp::chapter_knowledge::ChapterKnowledgeResp;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::chapter_knowledge;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 关联章节小节和知识点小类
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<CreateChapterKnowledgeReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<i32> {
    ApiResponse::response(chapter_knowledge::add(&app_state, req.into_inner()).await)
}

// 通过菜单标识获取关联详情-章节小节或者知识点小类标识
#[get("/list/{chapter_or_knowledge_id}")]
pub async fn list(
    app_state: web::Data<AppState>,
    path: web::Path<(i32,)>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<Vec<ChapterKnowledgeResp>> {
    ApiResponse::response(chapter_knowledge::list(&app_state, path.into_inner().0).await)
}

// 解除绑定关系
#[post("/remove")]
pub async fn remove(
    app_state: web::Data<AppState>,
    req: web::Json<RemoveChapterKnowledgeReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(chapter_knowledge::remove(&app_state, req.into_inner()).await)
}
