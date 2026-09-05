use crate::api::req::question::{
    CreateQuestionReq, DeleteReq, OriginalReq, QuestionListReq, QuestionSimilarListReq,
};
use crate::api::resp::question::{QuestionInfoResp, QuestionListResp};
use crate::app::conf::AppState;
use crate::middleware::user::UserInfo;
use crate::service::question;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 添加题目请求

// 添加题目
#[post("/add")]
pub async fn add(
    app_state: web::Data<AppState>,
    req: web::Json<CreateQuestionReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(question::add(&app_state, req.into_inner(), user_info).await)
}

#[get("/info/{id}")]
pub async fn info(
    app_state: web::Data<AppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<QuestionInfoResp> {
    ApiResponse::response(question::info(&app_state, path.into_inner().0).await)
}

// 题目列表
#[post("/list")]
pub async fn list(
    app_state: web::Data<AppState>,
    req: web::Json<QuestionListReq>,
    user_info: Option<UserInfo>,
) -> ApiResponse<QuestionListResp> {
    ApiResponse::response(question::list(&app_state, req.into_inner(), user_info).await)
}

#[post("/similar")]
pub async fn similar(
    app_state: web::Data<AppState>,
    req: web::Json<QuestionSimilarListReq>,
) -> ApiResponse<QuestionListResp> {
    ApiResponse::response(question::similar(&app_state, req.into_inner()).await)
}

// 课本原题标识
#[post("/original")]
pub async fn original(
    app_state: web::Data<AppState>,
    req: web::Json<OriginalReq>,
) -> ApiResponse<QuestionInfoResp> {
    ApiResponse::response(question::original(&app_state, req.into_inner()).await)
}

#[post("/delete")]
pub async fn delete(
    app_state: web::Data<AppState>,
    req: web::Json<DeleteReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(question::delete(&app_state, req.into_inner(), user_info).await)
}
