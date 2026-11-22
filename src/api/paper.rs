use crate::api::req::paper::DeleteReq;
use crate::api::req::paper::PaperAddReq;
use crate::api::req::paper::PaperListReq;
use crate::api::resp::paper::PaperListResp;
use crate::api::resp::paper::{CountPaperResp, PaperResp};
use crate::app::conf::WebAppState;
use crate::middleware::user::UserInfo;
use crate::service::paper;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

// 上传试卷
#[post("/upload/add")]
pub async fn upload_add(
    app_state: web::Data<WebAppState>,
    req: web::Json<PaperAddReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paper::upload_add(&app_state, req.into_inner(), user_info).await)
}

// 统计我的试卷数量
#[get("/count")]
pub async fn count(
    app_state: web::Data<WebAppState>,
    user_info: UserInfo,
) -> ApiResponse<Vec<CountPaperResp>> {
    ApiResponse::response(paper::count(&app_state, user_info).await)
}

#[post("/list")]
pub async fn list(
    app_state: web::Data<WebAppState>,
    req: web::Json<PaperListReq>,
    user_info: Option<UserInfo>,
) -> ApiResponse<PaperListResp> {
    ApiResponse::response(paper::list(&app_state, req.into_inner(), user_info).await)
}

// 查看试卷详情
#[get("/info/{id}")]
pub async fn info(
    app_state: web::Data<WebAppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<PaperResp> {
    ApiResponse::response(paper::info(&app_state, path.into_inner().0).await)
}

#[post("/delete")]
pub async fn delete(
    app_state: web::Data<WebAppState>,
    req: web::Json<DeleteReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(paper::delete(&app_state, req.into_inner(), user_info).await)
}
