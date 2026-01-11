use crate::AppConfig;
use crate::api::chapter_knowledge::{
    ChapterKnowledgeIdsReq, ChapterKnowledgeResp, CreateChapterKnowledgeReq,
    RemoveChapterKnowledgeReq,
};
use crate::api::textbook::TextbookResp;
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;
use crate::service::textbook;
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
            "当前选择的章节和知识点已关联过",
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
pub async fn info_by_chapter_or_knowledge(
    app_conf: web::Data<AppConfig>,
    id: i32,
) -> Result<ChapterKnowledgeResp, Error> {
    let row = ChapterKnowledge::find_by_chapter_or_knowledge_id(&app_conf.get_ref().db, id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if let Some(item) = row {
        Ok(to_resp(item))
    } else {
        Err(Error::new(ErrorKind::Other, "未做章节和知识点关联"))
    }
}

// 绑定关联关系
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateChapterKnowledgeReq,
) -> Result<ChapterKnowledgeResp, Error> {
    check_unique(&app_conf.get_ref().db, &req).await?;

    let row = ChapterKnowledge::insert(&app_conf.get_ref().db, &req)
        .await
        .map_err(|err| {
            error!("error adding chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "添加失败")
        })?;

    Ok(to_resp(row))
}

// 通过章节小节获取知识点类信息
pub async fn get_by_chapter(
    app_conf: web::Data<AppConfig>,
    req: ChapterKnowledgeIdsReq,
) -> Result<Vec<TextbookResp>, Error> {
    if req.ids.is_empty() {
        return Ok(vec![]);
    }

    let rows = ChapterKnowledge::find_by_chapter_ids(&app_conf.get_ref().db, req.ids)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let knowledge_ids: Vec<i32> = rows.iter().map(|row| row.knowledge_id).collect();
    let res = textbook::info_list_by_ids(app_conf, knowledge_ids).await?;
    Ok(res)
}

// 通过知识点小类获取章节信息
pub async fn get_by_knowledge(
    app_conf: web::Data<AppConfig>,
    req: ChapterKnowledgeIdsReq,
) -> Result<Vec<TextbookResp>, Error> {
    if req.ids.is_empty() {
        return Ok(vec![]);
    }

    let rows = ChapterKnowledge::find_by_knowledge_ids(&app_conf.get_ref().db, req.ids)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let chapter_ids: Vec<i32> = rows.iter().map(|row| row.chapter_id).collect();
    let res = textbook::info_list_by_ids(app_conf, chapter_ids).await?;
    Ok(res)
}

// 解除关联关系
pub async fn remove(
    app_conf: web::Data<AppConfig>,
    req: RemoveChapterKnowledgeReq,
) -> Result<bool, Error> {
    let req_id: i32 = req.id;
    if req_id <= 0 {
        return Err(Error::new(ErrorKind::Other, "删除数据标识为空"));
    }

    // 如果有题型关联就不能解除了
    let rows = QuestionCate::find_all_by_related_ids(&app_conf.get_ref().db, vec![req_id])
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if !rows.is_empty() {
        return Err(Error::new(ErrorKind::Other, "已关联了题型, 不能解除关联"));
    }

    let res = ChapterKnowledge::delete_by_chapter_or_knowledge_id(&app_conf.get_ref().db, req_id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            Error::new(ErrorKind::Other, "删除失败")
        })?;

    Ok(res > 0)
}
