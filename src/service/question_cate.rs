use crate::api::req::question_cate::CreateQuestionCateReq;
use crate::api::resp::question_cate::QuestionCateResp;
use crate::app::conf::AppState;
use crate::model::question::Question;
use crate::model::question_cate::QuestionCate;
use crate::util::error::AppError;
use tracing::error;

// 题型列表
pub async fn list(
    app_state: &AppState,
    related_id: i32,
) -> Result<Vec<QuestionCateResp>, AppError> {
    let db = &app_state.db;

    let rows = QuestionCate::find_all_by_related_ids(db, vec![related_id])
        .await
        .map_err(|err| {
            error!("error finding question cat: {}", err);
            AppError::db_error("题型查询失败")
        })?;

    let res: Vec<QuestionCateResp> = rows.into_iter().map(Into::into).collect();

    Ok(res)
}

// 添加题型
pub async fn add(app_state: &AppState, req: CreateQuestionCateReq) -> Result<i32, AppError> {
    let row_id = QuestionCate::save(&app_state.db, req)
        .await
        .map_err(|err| {
            error!("error adding question: {}", err);
            AppError::db_error("题型添加失败")
        })?;

    Ok(row_id)
}

// 删除题型
pub async fn remove(app_state: &AppState, id: i32) -> Result<bool, AppError> {
    let db = &app_state.db;

    // 关联题目后就不允许删除了
    let exist = Question::exist_by_cate_id(db, id).await.map_err(|err| {
        error!("error finding exists question: {}", err);
        AppError::db_error("题型查询失败")
    })?;
    if exist {
        return Err(AppError::permission_denied("题型已关联题目, 不允许删除"));
    }

    let row = QuestionCate::delete(db, id).await.map_err(|err| {
        error!("error deleting question: {}", err);
        AppError::db_error("题目删除失败")
    })?;

    Ok(row > 0)
}
