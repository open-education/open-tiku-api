use crate::api::resp::city::CityResp;
use crate::app::conf::WebAppState;
use crate::model::city::City;
use crate::util::error::AppError;
use tracing::error;

pub async fn list(app_state: &WebAppState) -> Result<Vec<CityResp>, AppError> {
    let db = &app_state.db;

    let rows = City::list(db).await.map_err(|err| {
        error!("city list err: {}", err);
        AppError::db_error("城市信息查询出错")
    })?;

    let res = rows.into_iter().map(Into::into).collect();

    Ok(res)
}
