use crate::api::req::question_cate::{CreateQuestionCateReq, QuestionCateListReq};
use crate::api::resp::question_cate::{QuestionCateListResp, QuestionCateResp};
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::question_cate;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 添加题型
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<CreateQuestionCateReq>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<i32> {
    ApiResponse::response(question_cate::add(&app_state, req.into_inner()).await)
}

// 题型列表 - 通过章节或者考点标识
#[get("/list/{related_id}")]
pub async fn list(
    app_state: web::Data<AppState>,
    path: web::Path<(i32,)>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<Vec<QuestionCateResp>> {
    ApiResponse::response(question_cate::list(&app_state, path.into_inner().0).await)
}

// 根据题型标识 question_cate_id 获取所有的父级菜单列表
#[post("/list/all")]
pub async fn list_all(
    app_state: web::Data<AppState>,
    req: web::Json<QuestionCateListReq>,
) -> ApiResponse<QuestionCateListResp> {
    ApiResponse::response(question_cate::list_all(&app_state, req.into_inner()).await)
}

// 删除题型
#[get("/remove/{id}")]
pub async fn remove(
    app_state: web::Data<AppState>,
    path: web::Path<(i32,)>,
    _user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(question_cate::remove(&app_state, path.into_inner().0).await)
}
