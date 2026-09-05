use crate::api::req::other_dict::CreateTextbookDictReq;
use crate::api::resp::other_dict::TextbookDictResp;
use crate::app::conf::AppState;
use crate::model::other_dict::TextbookDict;
use crate::util::error::AppError;
use tracing::error;

// 添加字典
pub async fn add(app_state: &AppState, req: CreateTextbookDictReq) -> Result<i32, AppError> {
    let db = &app_state.db;

    // 新增时需要判重
    if req.id.is_none() {
        let res =
            TextbookDict::find_by_unique(db, req.textbook_id, &req.type_code, &req.item_value)
                .await
                .map_err(|e| {
                    error!("error finding unique textbook item: {}", e);
                    AppError::db_error("字典查询失败")
                })?;
        if res.is_some() {
            return Err(AppError::business_error("字典已经存在"));
        }
    }

    let id = TextbookDict::save(db, req).await.map_err(|e| {
        error!("error adding unique textbook item: {}", e);
        AppError::db_error("字典新增失败")
    })?;

    Ok(id)
}

// 根据类型获取字典列表
pub async fn get_list(
    app_state: &AppState,
    textbook_id: i32,
    type_code: String,
) -> Result<Vec<TextbookDictResp>, AppError> {
    let db = &app_state.db;

    let rows = TextbookDict::find_by_textbook_and_type(db, textbook_id, &type_code)
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            AppError::db_error("字典列表查询失败")
        })?;
    let res: Vec<TextbookDictResp> = rows.into_iter().map(Into::into).collect();

    Ok(res)
}

// 删除字典
pub async fn delete(app_state: &AppState, id: i32) -> Result<bool, AppError> {
    //todo 被使用的字典不能删除, 字典id在题目题目类型和标签中

    let row = TextbookDict::delete(&app_state.db, id).await.map_err(|e| {
        error!("error deleting unique textbook item: {}", e);
        AppError::db_error("字典删除失败")
    })?;

    Ok(row > 0)
}
