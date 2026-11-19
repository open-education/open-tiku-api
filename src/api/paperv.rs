use crate::app::conf::WebAppState;
use crate::middleware::user::UserInfo;

use crate::service::paperv;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};

use crate::api::req::paperv::{
    DeleteReq, GenPaperPreviewReq, PaperGenReq, PaperListReq, TopPaperReq,
};
use crate::api::resp::paperv::{CommonPaperResp, GenPaperResp, PaperListResp, TopPaperResp};

// 试卷相关操作

// 添加精选试卷
#[post("/top/add")]
pub async fn top_add(
    app_state: web::Data<WebAppState>,
    req: web::Json<TopPaperReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paperv::top_add(&app_state, req.into_inner(), user_info).await)
}

// 查看精选试卷详情
#[get("/top/info/{id}")]
pub async fn top_info(
    app_state: web::Data<WebAppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<TopPaperResp> {
    ApiResponse::response(paperv::top_info(&app_state, path.into_inner().0).await)
}

#[post("/list")]
pub async fn list(
    app_state: web::Data<WebAppState>,
    req: web::Json<PaperListReq>,
    user_info: Option<UserInfo>,
) -> ApiResponse<PaperListResp> {
    ApiResponse::response(paperv::list(&app_state, req.into_inner(), user_info).await)
}

#[get("/latest/{paperType}/{count}")]
pub async fn latest(
    app_state: web::Data<WebAppState>,
    path: web::Path<(i16, i64)>,
) -> ApiResponse<Vec<CommonPaperResp>> {
    ApiResponse::response(paperv::latest(&app_state, path.into_inner()).await)
}

#[post("/gen/preview")]
pub async fn preview(
    app_state: web::Data<WebAppState>,
    req: web::Json<GenPaperPreviewReq>,
    user_info: UserInfo,
) -> ApiResponse<GenPaperResp> {
    ApiResponse::response(paperv::preview(&app_state, req.into_inner(), user_info).await)
}

// 保存试卷
#[post("/gen/add")]
pub async fn gen_add(
    app_state: web::Data<WebAppState>,
    req: web::Json<PaperGenReq>,
    user_info: UserInfo,
) -> ApiResponse<i64> {
    ApiResponse::response(paperv::gen_add(&app_state, req.into_inner(), user_info).await)
}

#[get("/gen/info/{id}")]
pub async fn gen_info(
    app_state: web::Data<WebAppState>,
    path: web::Path<(i64,)>,
) -> ApiResponse<GenPaperResp> {
    ApiResponse::response(paperv::gen_info(&app_state, path.into_inner().0).await)
}

#[post("/delete")]
pub async fn delete(
    app_state: web::Data<WebAppState>,
    req: web::Json<DeleteReq>,
    user_info: UserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(paperv::delete(&app_state, req.into_inner(), user_info).await)
}
