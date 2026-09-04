use crate::api::req::task::{TaskAddReq, TaskListReq};
use crate::api::resp::task::TaskListResp;
use crate::app::conf::AppState;
use crate::middleware::user::UserInfo;
use crate::model::task::Task;
use crate::util::error::AppError;
use tracing::error;

// 添加任务
pub async fn add(
    app_state: &AppState,
    req: TaskAddReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let db = &app_state.db;

    let row_id = Task::insert(db, req, user_info.user_id)
        .await
        .map_err(|e| {
            error!("task add err: {:?}", e);
            AppError::db_error("任务添加失败")
        })?;

    Ok(row_id)
}

pub async fn list(app_state: &AppState, req: TaskListReq) -> Result<TaskListResp, AppError> {
    let db = &app_state.db;

    // 1. 查询总数
    let total = Task::count_by_cate(db, req.question_cate_id, 1, req.task_type)
        .await
        .map_err(|e| {
            error!("task count by id err: {:?}", e);
            AppError::db_error("任务计数查询失败")
        })?;

    // 2. 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(TaskListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 3. 查询列表
    let list_data = Task::list_by_cate(
        db,
        req.question_cate_id,
        1,
        req.task_type,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("task list by id err: {:?}", e);
        AppError::db_error("任务列表查询失败")
    })?;

    // 4. 转换并返回
    Ok(TaskListResp {
        list: list_data.into_iter().map(Into::into).collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
