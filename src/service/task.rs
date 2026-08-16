use crate::api::task::{TaskAddReq, TaskInfoResp, TaskListReq, TaskListResp};
use crate::app::config::AppState;
use crate::middleware::user::UserInfo;
use crate::model::task::{Task, TaskStatus};
use crate::util::error::AppError;
use crate::util::local::to_local_datetime;
use actix_web::web;
use log::error;

// 添加任务
pub async fn add(
    app_state: web::Data<AppState>,
    req: TaskAddReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let db = &app_state.get_ref().db;

    let row_id = Task::insert(
        db,
        req.question_cate_id,
        req.task_type,
        &req.name,
        user_info.user_id,
        &req.url,
        &req.email,
        req.textbook_id,
    )
    .await
    .map_err(|e| {
        error!("task add err: {:?}", e);
        AppError::db_error("任务添加失败")
    })?;

    Ok(row_id)
}

fn to_base_resp(row: &Task) -> TaskInfoResp {
    TaskInfoResp {
        id: row.id,
        question_cate_id: row.question_cate_id,
        task_type: 0,
        name: row.name.clone(),
        author: "admin".to_string(),
        status: row.status,
        status_desc: TaskStatus::desc(row.status).to_string(),
        email: row.email.clone(),
        result: row.result.clone(),
        created_at: to_local_datetime(row.created_at),
        updated_at: to_local_datetime(row.updated_at),
    }
}

pub async fn list(
    app_state: web::Data<AppState>,
    req: TaskListReq,
) -> Result<TaskListResp, AppError> {
    let db = &app_state.db;

    // 1. 查询总数
    let total = Task::count_by_cate(db, req.question_cate_id, 1, req.task_type)
        .await
        .map_err(|e| {
            error!("task count by id err: {:?}", e);
            AppError::db_error("任务计数查询失败")
        })?;

    if total == 0 {
        return Ok(TaskListResp {
            list: vec![],
            page_no: 1,
            page_size: 10,
            total,
        });
    }

    // 2. 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;

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
        list: list_data
            .into_iter()
            .map(|row| to_base_resp(&row))
            .collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
