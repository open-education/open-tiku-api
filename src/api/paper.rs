use crate::api::req::paper::PaperAddReq;
use crate::api::req::paper::PaperListReq;
use crate::api::resp::paper::CountPaperResp;
use crate::api::resp::paper::PaperListResp;
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
