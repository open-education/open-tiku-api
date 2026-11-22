use crate::api::resp::city::CityResp;
use crate::app::conf::WebAppState;
use crate::constant::cache::CITY_CACHE_PREFIX;
use crate::model::city::City;
use crate::util::cache;
use crate::util::error::AppError;
use std::time::Duration;
use tracing::error;

pub async fn list(app_state: &WebAppState) -> Result<Vec<CityResp>, AppError> {
    let cache_key = format!("{CITY_CACHE_PREFIX}:list");
    match cache::get::<Vec<CityResp>>(&app_state.sqlite, &cache_key).await {
        Ok(resp) => return Ok(resp),
        Err(err) => {
            error!("Get city list all cache key: {cache_key}, msg: {}", err.msg);
        }
    }

    let db = &app_state.db;

    let rows = City::list(db).await.map_err(|err| {
        error!("city list err: {}", err);
        AppError::db_error("城市信息查询出错")
    })?;

    let res = rows.into_iter().map(Into::into).collect();

    cache::set::<Vec<CityResp>>(
        &app_state.sqlite,
        &cache_key,
        &res,
        Duration::from_hours(24),
    )
    .await;

    Ok(res)
}
