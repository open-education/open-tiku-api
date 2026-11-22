use crate::api::req::custom::{AddReq, ListReq};
use crate::api::resp::custom::CustomResp;
use crate::app::conf::WebAppState;
use crate::constant::meta;
use crate::enums::custom::{CustomType, PageSource, UserSource};
use crate::enums::user::RoleType;
use crate::middleware::user::UserInfo;
use crate::model::custom::Custom;
use crate::util::error::AppError;
use tracing::error;

// 保存个性化配置
pub async fn add(
    app_state: &WebAppState,
    req: AddReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let source_type = UserSource::from_i16(req.user_source)
        .ok_or_else(|| AppError::param_error("不受支持的来源类型"))?;
    let custom_type = CustomType::from_i16(req.custom_type)
        .ok_or_else(|| AppError::param_error("不受支持的个性化类型"))?;

    let db = &app_state.db;

    // 根据来源获取用户标识, 并检查权限
    let user_id = match source_type {
        UserSource::Person => user_info.user_id,
        UserSource::System => {
            // 必须是教师用户可设置
            if user_info.role != RoleType::Teacher as i16 {
                return Err(AppError::permission_denied("只有教师用户可设置系统信息"));
            }
            meta::SYSTEM_USER_ID
        }
    };

    match custom_type {
        CustomType::Textbook => {
            let textbook = req.textbook.unwrap_or_default();
            let row_id = Custom::save(db, req.id, user_id, custom_type as i16, textbook)
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
    req: ListReq,
    user_info: Option<UserInfo>,
) -> Result<Vec<CustomResp>, AppError> {
    PageSource::from_i16(req.page_source)
        .ok_or_else(|| AppError::param_error("不受支持的页面类型"))?;

    let user_source = UserSource::from_i16(req.user_source)
        .ok_or_else(|| AppError::param_error("不受支持的来源类型"))?;

    let user_id = match user_info {
        Some(user) => user.user_id,
        None => meta::SYSTEM_USER_ID, // 无登录信息时使用系统配置
    };

    let db = &app_state.db;

    let rows = match user_source {
        UserSource::Person => {
            let mut rows = Custom::list(db, user_id).await.map_err(|e| {
                error!("list custom row err {}", e);
                AppError::db_error("查询个性化信息出错")
            })?;

            // 如果个人用户未配置则使用系统配置
            if req.page_source == PageSource::Default as i16 && rows.is_empty() {
                rows = Custom::list(db, meta::SYSTEM_USER_ID).await.map_err(|e| {
                    error!("list custom row err {}", e);
                    AppError::db_error("查询系统默认个性化信息出错")
                })?;
            }

            rows
        }
        UserSource::System => Custom::list(db, meta::SYSTEM_USER_ID).await.map_err(|e| {
            error!("list custom row err {}", e);
            AppError::db_error("查询系统默认个性化信息出错")
        })?,
    };

    let res: Vec<CustomResp> = rows.into_iter().map(Into::into).collect();

    Ok(res)
}
