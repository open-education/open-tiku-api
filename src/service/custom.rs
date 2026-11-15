use crate::api::req::custom::CustomReq;
use crate::api::resp::custom::CustomResp;
use crate::app::conf::WebAppState;
use crate::enums::custom::CustomType;
use crate::middleware::user::UserInfo;
use crate::model::custom::Custom;
use crate::util::error::AppError;
use tracing::error;

// 保存个性化配置
pub async fn add(
    app_state: &WebAppState,
    req: CustomReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let custom_type = CustomType::from_i16(req.custom_type)
        .ok_or_else(|| AppError::param_error("不受支持的个性化类型"))?;

    let db = &app_state.db;

    match custom_type {
        CustomType::Textbook => {
            let textbook = req.textbook.unwrap_or_default();
            let row_id = Custom::save(db, req.id, user_info.user_id, custom_type as i16, textbook)
                .await
                .map_err(|e| {
                    error!("save custom row err {}", e);
                    AppError::db_error("保存个性化信息出错")
                })?;
            Ok(row_id)
        }
    }
}

// 个性化配置列表
pub async fn list(
    app_state: &WebAppState,
    user_info: Option<UserInfo>,
) -> Result<Vec<CustomResp>, AppError> {
    // 用户未登录时返回空
    if user_info.is_none() {
        return Ok(Vec::new());
    }

    let user = user_info.ok_or_else(|| AppError::business_error("用户信息获取异常"))?;

    let db = &app_state.db;

    let rows = Custom::list(db, user.user_id).await.map_err(|e| {
        error!("list custom row err {}", e);
        AppError::db_error("查询个性化信息出错")
    })?;

    let res: Vec<CustomResp> = rows.into_iter().map(Into::into).collect();

    Ok(res)
}
