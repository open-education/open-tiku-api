use crate::api::req::edit::CommonEditStatusReq;
use crate::app::conf::AppState;
use crate::enums::paper::PaperStatus;
use crate::enums::question::QuestionStatus;
use crate::middleware::user::TeacherUserInfo;
use crate::model::paper::Paper;
use crate::model::question::Question;
use crate::util::error::AppError;
use tracing::error;

// 更新题目状态
pub async fn question_status(
    app_state: &AppState,
    req: CommonEditStatusReq,
    user_info: TeacherUserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("题目标识不存在"));
    }

    let db = &app_state.db;

    let question = Question::find_by_id(db, req.id).await.map_err(|e| {
        error!("Select question id: {} err: {}", req.id, e);
        AppError::db_error("查询题目失败")
    })?;

    // 权限校验
    match req.status {
        // 作者提交审核
        s if s == QuestionStatus::Pending.as_i16() => {
            if question.author_id != user_info.0.user_id {
                return Err(AppError::permission_denied("只有题目作者才能提交审核"));
            }
        }
        // 审核结果（通过/拒绝/退回草稿）——均需教师权限
        s if s == QuestionStatus::Published.as_i16()
            || s == QuestionStatus::Rejected.as_i16()
            || s == QuestionStatus::Draft.as_i16() =>
        {
            // 拒绝时必须填写原因
            if s == QuestionStatus::Rejected.as_i16()
                && req.reject_reason.as_ref().is_none_or(|s| s.is_empty())
            {
                return Err(AppError::business_error("拒绝审核必须说明原因"));
            }
        }
        // 其他状态暂不支持
        _ => {
            return Err(AppError::business_error(
                format!("不支持的状态变更: {}", req.status).as_str(),
            ));
        }
    }

    let rows_affected = Question::update_status_by_id(
        db,
        req.id,
        req.status,
        user_info.0.user_id,
        req.reject_reason,
    )
    .await
    .map_err(|e| {
        error!("Update status by id: {} err: {}", req.id, e);
        AppError::db_error("更新题目失败")
    })?;

    Ok(rows_affected > 0)
}

// 更新试卷状态
pub async fn paper_status(
    app_state: &AppState,
    req: CommonEditStatusReq,
    user_info: TeacherUserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("试卷标识不存在"));
    }

    let db = &app_state.db;

    let paper = Paper::find_by_id(db, req.id)
        .await
        .map_err(|e| {
            error!("Select paper id: {} err: {}", req.id, e);
            AppError::db_error("查询试卷失败")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;

    // 权限校验
    match req.status {
        // 作者提交审核
        s if s == PaperStatus::Pending.as_i16() => {
            if paper.author_id != user_info.0.user_id {
                return Err(AppError::permission_denied("只有试卷作者才能提交审核"));
            }
        }
        // 审核结果（通过/拒绝/退回草稿）——均需教师权限
        s if s == PaperStatus::Published.as_i16()
            || s == PaperStatus::Rejected.as_i16()
            || s == PaperStatus::Draft.as_i16() =>
        {
            // 拒绝时必须填写原因
            if s == PaperStatus::Rejected.as_i16()
                && req.reject_reason.as_ref().is_none_or(|s| s.is_empty())
            {
                return Err(AppError::business_error("拒绝审核必须说明原因"));
            }
        }
        // 其他状态（如布置作业）暂不支持
        _ => {
            return Err(AppError::business_error(
                format!("不支持的状态变更: {}", req.status).as_str(),
            ));
        }
    }

    let rows_affected = Paper::update_status_by_id(
        db,
        req.id,
        req.status,
        user_info.0.user_id,
        req.reject_reason,
    )
    .await
    .map_err(|e| {
        error!("Update status by id: {} err: {}", req.id, e);
        AppError::db_error("更新试卷失败")
    })?;

    Ok(rows_affected > 0)
}
