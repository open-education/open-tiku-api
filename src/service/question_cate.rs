use crate::api::question_cate::{CreateQuestionCateReq, QuestionCateResp};
use crate::model::question_cate::QuestionCate;
use crate::AppConfig;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

fn to_resp(row: QuestionCate) -> QuestionCateResp {
    QuestionCateResp {
        id: row.id,
        related_id: row.related_id,
        label: row.label,
        key: row.key,
        sort_order: row.sort_order,
    }
}

// 题型列表
pub async fn list(
    app_conf: web::Data<AppConfig>,
    related_id: i32,
) -> Result<Vec<QuestionCateResp>, Error> {
    match QuestionCate::find_all_by_related_id(&app_conf.get_ref().db, related_id).await {
        Ok(rows) => {
            let mut res: Vec<QuestionCateResp> = vec![];
            for row in rows {
                res.push(to_resp(row));
            }
            Ok(res)
        }
        Err(err) => {
            error!("err:{:?}", err);
            Err(Error::new(ErrorKind::Other, "查询失败"))
        }
    }
}

// 添加题型
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateQuestionCateReq,
) -> Result<QuestionCateResp, Error> {
    match QuestionCate::insert(&app_conf.get_ref().db, req).await {
        Ok(res) => Ok(to_resp(res)),
        Err(err) => {
            error!("question cate add error: {:?}", err);
            Err(Error::new(ErrorKind::Other, "添加失败"))
        }
    }
}

// 删除题型
pub async fn remove(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    //todo 关联题目后就不允许删除了

    match QuestionCate::delete(&app_conf.get_ref().db, id).await {
        Ok(row) => Ok(row > 0),
        Err(err) => {
            error!("question cate remove error: {:?}", err);
            Err(Error::new(ErrorKind::Other, "删除失败"))
        }
    }
}
