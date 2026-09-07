use crate::api::req::other_dict::{CreateTextbookDictReq, DictListReq};
use crate::api::resp::other_dict::{DictListResp, TextbookDictResp};
use crate::app::conf::AppState;
use crate::enums::dict::TypeCode;
use crate::model::other_dict::TextbookDict;
use crate::model::question::{ExtIdReq, Question};
use crate::util::error::AppError;
use std::collections::HashMap;
use tracing::error;

// 添加字典
pub async fn add(app_state: &AppState, req: CreateTextbookDictReq) -> Result<i32, AppError> {
    let db = &app_state.db;

    TypeCode::from_str(&req.type_code)
        .ok_or_else(|| AppError::param_error("不受支持的字典类型"))?;

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

pub async fn list_all(app_state: &AppState, req: DictListReq) -> Result<DictListResp, AppError> {
    let db = &app_state.db;

    let rows = TextbookDict::find_by_textbook_id(db, req.textbook_id, req.codes)
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            AppError::db_error("查询教材字典出错")
        })?;

    let mut map: HashMap<String, Vec<TextbookDictResp>> = HashMap::new();
    for item in rows.into_iter() {
        map.entry(item.type_code.clone())
            .or_default()
            .push(item.into());
    }

    Ok(DictListResp { map })
}

// 删除字典
pub async fn delete(app_state: &AppState, id: i32) -> Result<bool, AppError> {
    let db = &app_state.db;

    let row = TextbookDict::find_by_id(db, id)
        .await
        .map_err(|e| {
            error!("error deleting unique textbook item: {}", e);
            AppError::db_error("字典查询出错")
        })?
        .ok_or_else(|| AppError::not_found("字典不存在"))?;
    let row_id = row
        .id
        .ok_or_else(|| AppError::business_error("字典数据不完整"))?;

    // 检查字典属于什么类型
    let type_code = TypeCode::from_str(&row.type_code)
        .ok_or_else(|| AppError::business_error("不支持的字典类型"))?;
    // 看该类型的字典是否关联了题目
    let ext_id_req = match type_code {
        TypeCode::QuestionType => ExtIdReq {
            type_id: Some(row_id),
            tag_ids: None,
            dimension_ids: None,
            level_id: None,
            scene_ids: None,
            mistake_tip_ids: None,
        },
        TypeCode::QuestionTag => ExtIdReq {
            type_id: None,
            tag_ids: Some(vec![row_id]),
            dimension_ids: None,
            level_id: None,
            scene_ids: None,
            mistake_tip_ids: None,
        },
        TypeCode::QuestionDimension => ExtIdReq {
            type_id: None,
            tag_ids: None,
            dimension_ids: Some(vec![row_id]),
            level_id: None,
            scene_ids: None,
            mistake_tip_ids: None,
        },
        TypeCode::QuestionLevel => ExtIdReq {
            type_id: None,
            tag_ids: None,
            dimension_ids: None,
            level_id: Some(row_id),
            scene_ids: None,
            mistake_tip_ids: None,
        },
        TypeCode::QuestionScene => ExtIdReq {
            type_id: None,
            tag_ids: None,
            dimension_ids: None,
            level_id: None,
            scene_ids: Some(vec![row_id]),
            mistake_tip_ids: None,
        },
        TypeCode::QuestionMistakeTip => ExtIdReq {
            type_id: None,
            tag_ids: None,
            dimension_ids: None,
            level_id: None,
            scene_ids: None,
            mistake_tip_ids: Some(vec![row_id]),
        },
    };
    let exist = Question::exists_by_ext_id(db, &ext_id_req)
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            AppError::db_error("检查题目是否存在出错")
        })?;
    if exist {
        return Err(AppError::business_error(
            "该标签下面已经维护有题目, 不允许删除",
        ));
    }

    let del_rows = TextbookDict::delete(db, id).await.map_err(|e| {
        error!("error deleting unique textbook item: {}", e);
        AppError::db_error("字典删除失败")
    })?;

    Ok(del_rows > 0)
}
