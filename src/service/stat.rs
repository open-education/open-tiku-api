use crate::api::resp::stat::{BoardResp, LatestQuestionResp};
use crate::app::conf::AppState;
use crate::model::question::Question;
use crate::util::error::AppError;
use sqlx::PgPool;
use tracing::error;

// 首页面板数据
pub async fn board(app_state: &AppState, limit: i16) -> Result<BoardResp, AppError> {
    if limit <= 0 || limit > 15 {
        return Err(AppError::param_error("非法的数量"));
    }

    let db = &app_state.db;

    // 最新上传题目
    let latest_questions = latest_question(db, limit).await?;

    Ok(BoardResp { latest_questions })
}

async fn latest_question(pool: &PgPool, limit: i16) -> Result<Vec<LatestQuestionResp>, AppError> {
    // 最新上传的题目, 暂时 status 没有索引, 这类查询区分度很低, 索引大部分可能都提高不了查询效率, 后面数据量上来后这类最新的列表实际上意义就不大了
    // 可以将这类最新的题目写入统计表存储固定大小, 永远保持最新的那几个题目即可
    let rows = Question::latest_question(pool, limit).await.map_err(|e| {
        error!("board find latest question err: {e}");
        AppError::db_error("获取最新上传题目出错")
    })?;

    Ok(rows.into_iter().map(Into::into).collect())
}
