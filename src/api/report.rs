use crate::api::req::report::{AddReq, ListReq};
use crate::api::resp::report::ReportListResp;
use crate::app::conf::WebAppState;
use crate::service::report;
use crate::util::response::ApiResponse;
use actix_web::{post, web};

#[post("/add")]
pub async fn add(app_state: web::Data<WebAppState>, req: web::Json<AddReq>) -> ApiResponse<i64> {
    ApiResponse::response(report::add(&app_state, req.into_inner()).await)
}

#[post("/list")]
pub async fn list(
    app_state: web::Data<WebAppState>,
    req: web::Json<ListReq>,
) -> ApiResponse<ReportListResp> {
    ApiResponse::response(report::list(&app_state, req.into_inner()).await)
}
