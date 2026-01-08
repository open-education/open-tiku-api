use crate::api::textbook_dict::{CreateTextbookDictReq, TextbookDictListReq, TextbookDictResp};
use crate::model::textbook_dict::TextbookDict;
use crate::AppConfig;
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
    }
}

// 添加字典
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateTextbookDictReq,
) -> Result<TextbookDictResp, Error> {
    match TextbookDict::find_by_unique(
        &app_conf.get_ref().db,
        req.textbook_id,
        &req.type_code,
        &req.item_value,
    )
    .await
    {
        Ok(res) => {
            if let Some(_) = res {
                return Err(Error::new(ErrorKind::Other, "字典已经存在"));
            }
        }
        Err(err) => {
            error!("textbook dict query err: {}", err);
            return Err(Error::new(ErrorKind::Other, "查询失败"));
        }
    }

    match TextbookDict::insert(&app_conf.get_ref().db, req).await {
        Ok(row) => Ok(to_resp(row)),
        Err(err) => {
            error!("textbook dict add err: {:?}", err);
            Err(Error::new(ErrorKind::Other, "新增失败"))
        }
    }
}

// 根据类型获取字典列表
pub async fn get_list(
    app_conf: web::Data<AppConfig>,
    req: TextbookDictListReq,
) -> Result<Vec<TextbookDictResp>, Error> {
    match TextbookDict::find_by_textbook_and_type(
        &app_conf.get_ref().db,
        req.textbook_id,
        &req.type_code,
    )
    .await
    {
        Ok(rows) => {
            let mut res: Vec<TextbookDictResp> = vec![];
            for row in rows {
                res.push(to_resp(row));
            }
            Ok(res)
        }
        Err(err) => {
            error!("textbook dict query err: {:?}", err);
            Err(Error::new(ErrorKind::Other, "查询失败"))
        }
    }
}

// 删除字典
pub async fn delete(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    match TextbookDict::delete(&app_conf.get_ref().db, id).await {
        Ok(row) => Ok(row > 0),
        Err(err) => {
            error!("textbook dict delete err: {:?}", err);
            Err(Error::new(ErrorKind::Other, "删除失败"))
        }
    }
}
