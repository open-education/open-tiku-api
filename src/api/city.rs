use crate::api::resp::city::CityResp;
use crate::app::conf::WebAppState;
use crate::service::city;
use crate::util::response::ApiResponse;
use actix_web::{get, web};

// 城市列表信息
#[get("/list")]
pub async fn list(app_state: web::Data<WebAppState>) -> ApiResponse<Vec<CityResp>> {
    ApiResponse::response(city::list(&app_state).await)
}
