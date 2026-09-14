use crate::api::resp::stat::BoardResp;
use crate::app::conf::AppState;
use crate::service::stat;
use crate::util::response::ApiResponse;
use actix_web::{get, web};

// 首页统计
#[get("board/{num}")]
pub async fn board(
    app_state: web::Data<AppState>,
    path: web::Path<(i16,)>,
) -> ApiResponse<BoardResp> {
    ApiResponse::response(stat::board(&app_state, path.into_inner().0).await)
}
