use crate::api::chapter_knowledge::{
    ChapterKnowledgeResp, CreateChapterKnowledgeReq, RemoveChapterKnowledgeReq,
};

use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;

use crate::AppConfig;
use actix_web::web;
use log::error;
use sqlx::PgPool;

use std::io::{Error, ErrorKind};

// 查询唯一绑定关系是否一存在
async fn check_unique(pool: &PgPool, req: &CreateChapterKnowledgeReq) -> Result<(), Error> {
    let res = ChapterKnowledge::find_unique(&pool, req.chapter_id, req.knowledge_id)
        .await
        .map_err(|err| {
            error!("add relation query err: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if res.is_none() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "当前选择的章节和知识点已存在关联关系, 无需重复关联",
        ))
    }
}

fn to_resp(row: ChapterKnowledge) -> ChapterKnowledgeResp {
    ChapterKnowledgeResp {
        id: Some(row.id),
        chapter_id: row.chapter_id,
        knowledge_id: row.knowledge_id,
    }
}

// 通过章节或者知识点获取关联信息
pub async fn list(
    app_conf: web::Data<AppConfig>,
    id: i32,
) -> Result<Vec<ChapterKnowledgeResp>, Error> {
    let rows = ChapterKnowledge::find_by_ids(&app_conf.get_ref().db, vec![id])
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if !rows.is_empty() {
        Ok(rows.into_iter().map(to_resp).collect())
    } else {
        Ok(Vec::new())
    }
}

// 绑定关联关系
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateChapterKnowledgeReq,
) -> Result<i32, Error> {
    let db = &app_conf.get_ref().db;

    check_unique(db, &req).await?;

    let row_id = ChapterKnowledge::insert(db, &req).await.map_err(|err| {
        error!("error adding chapter knowledge: {}", err);
        Error::new(ErrorKind::Other, "添加失败")
    })?;

    Ok(row_id)
}

// 解除关联关系
pub async fn remove(
    app_conf: web::Data<AppConfig>,
    req: RemoveChapterKnowledgeReq,
) -> Result<bool, Error> {
    let chapter_id: i32 = req.chapter_id;
    if chapter_id <= 0 {
        return Err(Error::new(ErrorKind::Other, "章节标识为空"));
    }

    let knowledge_id: i32 = req.knowledge_id;
    if knowledge_id <= 0 {
        return Err(Error::new(ErrorKind::Other, "考点标识为空"));
    }

    let db = &app_conf.get_ref().db;

    // 查询关联记录
    let relation_row = ChapterKnowledge::find_unique(&db, chapter_id, knowledge_id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "考点章节关联查询失败")
        })?;
    if relation_row.is_none() {
        return Err(Error::new(
            ErrorKind::Other,
            "章节/考点没有关联关系, 无需解绑",
        ));
    }

    // 如果有题型关联就不能解除了, 后续如果需要放开重新绑定再处理
    let rows = QuestionCate::find_all_by_related_ids(db, vec![relation_row.unwrap().id])
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if !rows.is_empty() {
        return Err(Error::new(ErrorKind::Other, "已关联了题型, 不能解除关联"));
    }

    let res = ChapterKnowledge::delete_by_chapter_knowledge_id(db, chapter_id, knowledge_id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "删除失败")
        })?;

    Ok(res > 0)
}
