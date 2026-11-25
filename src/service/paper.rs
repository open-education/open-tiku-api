use crate::api::req::paper::{DeleteReq, PaperAddReq, PaperListReq, PendingReq, UndoReq};
use crate::api::resp::paper::{CountPaperResp, PaperListResp, PaperResp};
use crate::app::conf::WebAppState;
use crate::enums::paper::{
    ExamType, PaperDifficultyType, PaperLevelType, PaperStatus, PaperType, UploadFileType,
};
use crate::enums::user::PaperPageSource;
use crate::middleware::user::UserInfo;
use crate::model::paper::{Paper, UpdatePaperReq};
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tracing::error;

pub async fn upload_add(
    app_state: &WebAppState,
    req: PaperAddReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    // 检查参数
    check_paper_add_req(&req)?;

    let db = &app_state.db;

    // 只能编辑自己的试卷
    if let Some(id) = req.id {
        validate_is_allow_edit(db, id, user_info.user_id).await?;
    }

    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed begin transaction: {e}");
        AppError::db_error("启动事务失败")
    })?;

    let row_id = Paper::save(&mut tx, &req, user_info.user_id)
        .await
        .map_err(|err| {
            error!("Upload paper add err: {err}");
            AppError::db_error("上传试卷保存出错")
        })?;

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed commit transaction: {e}");
        AppError::db_error("提交事务失败")
    })?;

    Ok(row_id)
}

// 以下均为上传试卷时的检查条件, 生成试卷等后续处理
fn check_paper_add_req(req: &PaperAddReq) -> Result<(), AppError> {
    // 类型相关检查
    PaperType::from_i16(req.paper_type)
        .ok_or_else(|| AppError::param_error("不受支持的试卷类型"))?;
    UploadFileType::from_i16(req.file_type)
        .ok_or_else(|| AppError::param_error("不受支持的文件 类型"))?;
    ExamType::from_i16(req.exam_type).ok_or_else(|| AppError::param_error("不受支持的考试类型"))?;
    PaperLevelType::from_i16(req.level_type)
        .ok_or_else(|| AppError::param_error("不受支持的考试级别"))?;
    PaperDifficultyType::from_i16(req.difficulty_type)
        .ok_or_else(|| AppError::param_error("不受支持的难度级别"))?;

    let status =
        PaperStatus::from_i16(req.status).ok_or_else(|| AppError::param_error("试卷状态错误"))?;
    // 只有草稿可以添加
    if status != PaperStatus::Draft {
        return Err(AppError::business_error("只有草稿中的数据可以添加"));
    }

    if req.textbook_path.is_empty() {
        return Err(AppError::param_error("菜单不能为空"));
    }
    if req.file_list.as_ref().is_none_or(Vec::is_empty) {
        return Err(AppError::param_error("上传文件不能为空"));
    }
    if req.title.is_empty() {
        return Err(AppError::param_error("试卷名称不能为空"));
    }
    if req.year.is_empty() {
        return Err(AppError::param_error("学年不能为空"));
    }

    Ok(())
}

// 只允许编辑自己的试卷
async fn validate_is_allow_edit(db: &PgPool, id: i64, user_id: i64) -> Result<(), AppError> {
    let has_paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("validate paper err: {err}");
            AppError::db_error("查询试卷信息错误")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if has_paper.author_id != user_id {
        return Err(AppError::permission_denied("只允许编辑自己的试卷"));
    }

    Ok(())
}

pub async fn count(
    app_state: &WebAppState,
    user_info: UserInfo,
) -> Result<Vec<CountPaperResp>, AppError> {
    let db = &app_state.db;

    let rows = Paper::find_count_by_author_id(db, user_info.user_id)
        .await
        .map_err(|err| {
            error!("question count by id err: {err}");
            AppError::db_error("统计用户试卷状态计数出错")
        })?;

    let result: Vec<CountPaperResp> = rows.into_iter().map(Into::into).collect();

    Ok(result)
}

pub async fn list(
    app_state: &WebAppState,
    mut req: PaperListReq,
    user_info: Option<UserInfo>,
) -> Result<PaperListResp, AppError> {
    let db = &app_state.db;

    // 页面来源
    let req_source = PaperPageSource::from_str(&req.source)
        .ok_or_else(|| AppError::param_error("不清楚的查询来源"))?;

    if req.textbook_path.is_empty() {
        return Err(AppError::param_error("菜单不能为空"));
    }

    // 我的试卷等时需要登录
    let (author_id, status) = if req_source == PaperPageSource::List {
        (None, Some(PaperStatus::Published as i16))
    } else {
        let user_info = user_info.ok_or_else(|| AppError::permission_denied("需要登录方能访问"))?;
        // 目前我的试卷需要指定作者, 其它情况下暂时不需要指定作者, 后续审核查询条件重新指定
        // 我的题目和审核状态由前端自己指定
        if req_source == PaperPageSource::MyPaper {
            (Some(user_info.user_id), req.status)
        } else {
            (None, req.status)
        }
    };
    req.status = status;

    // 查询总数
    let total = Paper::count(db, &req, author_id).await.map_err(|e| {
        error!("paper list count err: {e}");
        AppError::db_error("试卷计数查询失败")
    })?;

    // 计算偏移量
    let offset: i64 = (req.page_no - 1) * req.page_size;
    if offset >= total {
        return Ok(PaperListResp {
            list: Vec::new(),
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 查询列表
    let rows = Paper::list(db, &req, author_id, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("paper list err: {e}");
            AppError::db_error("试卷列表查询失败")
        })?;

    let mut user_ids_set = HashSet::with_capacity(rows.len() * 2);

    for q in &rows {
        user_ids_set.insert(q.author_id);
        if let Some(approve_id) = q.approve_id
            && approve_id > 0
        {
            user_ids_set.insert(approve_id);
        }
    }

    let user_ids: Vec<i64> = user_ids_set.into_iter().collect();
    let user_map: HashMap<i64, String> = get_user_map(db, user_ids).await?;

    Ok(PaperListResp {
        list: to_list_resp(rows, &user_map),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

fn to_list_resp(rows: Vec<Paper>, user_map: &HashMap<i64, String>) -> Vec<PaperResp> {
    let mut list: Vec<PaperResp> = Vec::with_capacity(rows.len());
    for row in rows {
        let author_name = user_map.get(&row.author_id).cloned().unwrap_or_default();
        let approve_name = user_map
            .get(&row.approve_id.unwrap_or_default())
            .cloned()
            .unwrap_or_default();
        let info: PaperResp = (row, author_name, approve_name).into();
        list.push(info);
    }

    list
}

// 试卷详情
pub async fn info(app_state: &WebAppState, id: i64) -> Result<PaperResp, AppError> {
    let db = &app_state.db;

    let row = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select paper info by id: {id} error: {err}");
            AppError::db_error("查询试卷详情出错")
        })?
        .ok_or_else(|| AppError::business_error("试卷不存在"))?;

    Ok(row.into())
}

pub async fn delete(
    app_state: &WebAppState,
    req: DeleteReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("试卷标识为空"));
    }

    let db = &app_state.db;

    // 只允许删除自己的试卷
    let has_paper = Paper::find_by_id(db, req.id)
        .await
        .map_err(|err| {
            error!("Failed to find paper: {err}");
            AppError::db_error("试卷查询错误")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if has_paper.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只允许删除自己的试卷"));
    }

    // 只有草稿的试卷可以删除
    if has_paper.status != PaperStatus::Draft as i16 {
        return Err(AppError::business_error("只允许删除草稿中的试卷"));
    }

    let rows = Paper::delete(db, req.id).await.map_err(|err| {
        error!("paper delete by id err: {err}");
        AppError::db_error("删除失败")
    })?;

    Ok(rows >= 1)
}

pub async fn pending(
    app_state: &WebAppState,
    req: PendingReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("试卷标识不存在"));
    }

    let db = &app_state.db;

    let paper = Paper::find_by_id(db, req.id)
        .await
        .map_err(|e| {
            error!("Select paper id: {} err: {e}", req.id);
            AppError::db_error("查询试卷失败")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if paper.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只能撤回自己的试卷"));
    }
    if paper.status != PaperStatus::Draft as i16 {
        return Err(AppError::business_error("只有草稿中的试卷需要提交审核"));
    }

    let update_req: UpdatePaperReq = UpdatePaperReq {
        id: req.id,
        status: PaperStatus::Pending as i16,
        approve_id: 0,
        reject_reason: Some(String::new()),
        approve_at: None,
    };
    let rows_affected = Paper::update_status_by_id(db, update_req)
        .await
        .map_err(|e| {
            error!("Update status by id: {} err: {e}", req.id);
            AppError::db_error("更新试卷失败")
        })?;

    Ok(rows_affected > 0)
}

pub async fn undo(
    app_state: &WebAppState,
    req: UndoReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("试卷标识不存在"));
    }

    let db = &app_state.db;

    let paper = Paper::find_by_id(db, req.id)
        .await
        .map_err(|e| {
            error!("Select paper id: {} err: {e}", req.id);
            AppError::db_error("查询试卷失败")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if paper.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只能撤回自己的题目"));
    }
    if paper.status != PaperStatus::Published as i16 {
        return Err(AppError::business_error("只有已发布的试卷需要撤回"));
    }

    let update_req: UpdatePaperReq = UpdatePaperReq {
        id: req.id,
        status: PaperStatus::Draft as i16,
        approve_id: 0,
        reject_reason: Some(String::new()),
        approve_at: None,
    };
    let rows_affected = Paper::update_status_by_id(db, update_req)
        .await
        .map_err(|e| {
            error!("Update status by id: {} err: {e}", req.id);
            AppError::db_error("更新试卷失败")
        })?;

    Ok(rows_affected > 0)
}
