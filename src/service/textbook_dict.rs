use crate::AppConfig;
use crate::api::other_dict::{CreateTextbookDictReq, TextbookDictResp};
use crate::model::other_dict::TextbookDict;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

fn to_resp(row: TextbookDict) -> TextbookDictResp {
    TextbookDictResp {
        id: row.id,
        textbook_id: row.textbook_id,
        type_code: row.type_code,
        item_value: row.item_value,
        sort_order: row.sort_order,
        is_select: row.is_select,
    }
}

// 添加字典
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateTextbookDictReq,
) -> Result<TextbookDictResp, Error> {
    let db = &app_conf.get_ref().db;

    let res = TextbookDict::find_by_unique(db, req.textbook_id, &req.type_code, &req.item_value)
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;
    if res.is_some() {
        return Err(Error::new(ErrorKind::Other, "字典已经存在"));
    }

    let row = TextbookDict::insert(db, req).await.map_err(|e| {
        error!("error adding unique textbook item: {}", e);
        Error::new(ErrorKind::Other, "新增失败")
    })?;
    Ok(to_resp(row))
}

// 根据类型获取字典列表
pub async fn get_list(
    app_conf: web::Data<AppConfig>,
    textbook_id: i32,
    type_code: String,
) -> Result<Vec<TextbookDictResp>, Error> {
    let db = &app_conf.get_ref().db;

    let rows = TextbookDict::find_by_textbook_and_type(db, textbook_id, &type_code)
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;
    let res: Vec<TextbookDictResp> = rows.into_iter().map(to_resp).collect();

    Ok(res)
}

// 删除字典
pub async fn delete(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    let row = TextbookDict::delete(&app_conf.get_ref().db, id)
        .await
        .map_err(|e| {
            error!("error deleting unique textbook item: {}", e);
            Error::new(ErrorKind::Other, "删除失败")
        })?;

    Ok(row > 0)
}
