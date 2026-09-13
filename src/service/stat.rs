use crate::api::resp::stat::{BoardResp, LatestQuestionResp};
use crate::app::conf::AppState;
use crate::constant::meta::STAT_MAX_NUM;
use crate::model::question::Question;
use crate::model::stat::Stat;
use crate::util::error::AppError;
use crate::util::local::get_today;
use sqlx::PgPool;
use tracing::error;

// 首页面板数据
pub async fn board(app_state: &AppState, limit: i16) -> Result<BoardResp, AppError> {
    if limit <= 0 || limit > 15 {
        return Err(AppError::param_error("非法的数量"));
    }

    let date = get_today();
    let db = &app_state.db;

    let row = Stat::find_by_id(db, date)
        .await
        .map_err(|e| {
            error!("board find stat date row err {e}");
            AppError::db_error("获取统计数据出错")
        })?
        .unwrap_or_else(|| Stat {
            date: Some(date),
            ..Default::default()
        });

    // 最新上传题目
    let latest_questions = latest_question(db, &row.latest_question_ids.0).await?;

    Ok(BoardResp { latest_questions })
}

async fn latest_question(pool: &PgPool, ids: &[i64]) -> Result<Vec<LatestQuestionResp>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut rows = Question::find_simple_by_ids(pool, ids).await.map_err(|e| {
        error!("board find latest question err: {e}");
        AppError::db_error("获取最新上传题目出错")
    })?;
    rows.sort_by(|a, b| b.id.cmp(&a.id));

    Ok(rows.into_iter().map(Into::into).collect())
}

// 记录最新的题目
pub async fn add_lastest_question_id(pool: &PgPool, id: i64) -> Result<bool, AppError> {
    let date = get_today();
    let mut row = Stat::find_by_id(pool, date)
        .await
        .map_err(|e| {
            error!("get lastest question date err: {e}");
            AppError::db_error("查询当前日期统计数据出错")
        })?
        .unwrap_or_else(|| Stat {
            date: Some(date),
            ..Stat::default()
        });

    if row.latest_question_ids.0.len() >= STAT_MAX_NUM {
        if let Some(idx) = row
            .latest_question_ids
            .0
            .iter()
            .enumerate()
            .min_by_key(|&(_, &v)| v)
            .map(|(i, _)| i)
        {
            row.latest_question_ids.0.remove(idx);
        }
    }
    row.latest_question_ids.0.push(id);

    Stat::save(pool, row).await.map_err(|e| {
        error!("stat save latest question id err: {e}");
        AppError::db_error("最新题目统计信息增加出错")
    })?;

    Ok(true)
}
