use crate::AppConfig;
use crate::api::question_cate::{CreateQuestionCateReq, QuestionCateResp};
use crate::model::question_cate::QuestionCate;
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
    let rows = QuestionCate::find_all_by_related_ids(&app_conf.get_ref().db, vec![related_id])
        .await
        .map_err(|err| {
            error!("error finding question cat: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let mut res: Vec<QuestionCateResp> = vec![];
    for row in rows {
        res.push(to_resp(row));
    }

    Ok(res)
}

// 添加题型
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateQuestionCateReq,
) -> Result<QuestionCateResp, Error> {
    let res = QuestionCate::insert(&app_conf.get_ref().db, req)
        .await
        .map_err(|err| {
            error!("error adding question: {}", err);
            Error::new(ErrorKind::Other, "添加失败")
        })?;

    Ok(to_resp(res))
}

// 删除题型
pub async fn remove(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    //todo 关联题目后就不允许删除了

    let row = QuestionCate::delete(&app_conf.get_ref().db, id)
        .await
        .map_err(|err| {
            error!("error deleting question: {}", err);
            Error::new(ErrorKind::Other, "删除失败")
        })?;

    Ok(row > 0)
}
