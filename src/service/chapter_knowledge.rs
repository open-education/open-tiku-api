use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;

use crate::api::req::chapter_knowledge::{CreateChapterKnowledgeReq, RemoveChapterKnowledgeReq};
use crate::api::resp::chapter_knowledge::ChapterKnowledgeResp;
use crate::app::conf::AppState;
use crate::util::error::AppError;
use sqlx::PgPool;
use tracing::error;

// 查询唯一绑定关系是否一存在
async fn check_unique(pool: &PgPool, req: &CreateChapterKnowledgeReq) -> Result<(), AppError> {
    let res = ChapterKnowledge::find_unique(pool, req.chapter_id, req.knowledge_id)
        .await
        .map_err(|err| {
            error!("add relation query err: {}", err);
            AppError::db_error("章节/考点绑定关系查询失败")
        })?;

    if res.is_none() {
        Ok(())
    } else {
        Err(AppError::param_error(
            "当前选择的章节和知识点已存在关联关系, 无需重复关联",
        ))
    }
}

// 通过章节或者知识点获取关联信息
pub async fn list(app_state: &AppState, id: i32) -> Result<Vec<ChapterKnowledgeResp>, AppError> {
    let rows = ChapterKnowledge::find_by_ids(&app_state.db, vec![id])
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            AppError::db_error("绑定关系查询失败")
        })?;

    if !rows.is_empty() {
        Ok(rows.into_iter().map(Into::into).collect())
    } else {
        Ok(Vec::new())
    }
}

// 绑定关联关系
pub async fn add(app_state: &AppState, req: CreateChapterKnowledgeReq) -> Result<i32, AppError> {
    let db = &app_state.db;

    check_unique(db, &req).await?;

    let row_id = ChapterKnowledge::insert(db, &req).await.map_err(|err| {
        error!("error adding chapter knowledge: {}", err);
        AppError::db_error("绑定失败")
    })?;

    Ok(row_id)
}

// 解除关联关系
pub async fn remove(
    app_state: &AppState,
    req: RemoveChapterKnowledgeReq,
) -> Result<bool, AppError> {
    let chapter_id: i32 = req.chapter_id;
    if chapter_id <= 0 {
        return Err(AppError::param_error("章节标识为空"));
    }

    let knowledge_id: i32 = req.knowledge_id;
    if knowledge_id <= 0 {
        return Err(AppError::param_error("考点标识为空"));
    }

    let db = &app_state.db;

    // 查询关联记录
    let relation_row = ChapterKnowledge::find_unique(db, chapter_id, knowledge_id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            AppError::db_error("考点章节关联查询失败")
        })?;
    if relation_row.is_none() {
        return Err(AppError::param_error("章节/考点没有关联关系, 无需解绑"));
    }
    let relation_id = relation_row.unwrap().id;
    if relation_id != req.id {
        return Err(AppError::param_error("章节/考点关联关系不匹配, 无需解绑"));
    }

    // 如果有题型关联就不能解除了, 后续如果需要放开重新绑定再处理
    let rows = QuestionCate::find_all_by_related_ids(db, vec![relation_id])
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            AppError::db_error("绑定关系查询失败")
        })?;

    if !rows.is_empty() {
        return Err(AppError::business_error("已关联了题型, 不能解除关联"));
    }

    let res = ChapterKnowledge::delete_by_id(db, req.id)
        .await
        .map_err(|err| {
            error!("error fetching chapter knowledge: {}", err);
            AppError::db_error("删除失败")
        })?;

    Ok(res > 0)
}
