use crate::api::req::task::{TaskAddReq, TaskListReq};
use crate::api::resp::task::{TaskInfoResp, TaskListResp};
use crate::app::conf::AppState;
use crate::middleware::user::UserInfo;
use crate::model::task::Task;
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use std::collections::HashMap;
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

    let total = Task::count_by_cate(db, req.question_cate_id, req.task_type)
        .await
        .map_err(|e| {
            error!("task count by id err: {:?}", e);
            AppError::db_error("任务计数查询失败")
        })?;

    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(TaskListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    let list_data = Task::list_by_cate(
        db,
        req.question_cate_id,
        req.task_type,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("task list by id err: {:?}", e);
        AppError::db_error("任务列表查询失败")
    })?;

    // 作者名称补充
    let mut author_ids: Vec<i64> = list_data.iter().map(|task| task.author_id).collect();
    author_ids.sort_unstable();
    author_ids.dedup();

    let user_name_map: HashMap<i64, String> = get_user_map(db, author_ids).await?;

    let resp_list: Vec<TaskInfoResp> = list_data
        .into_iter()
        .map(|row| {
            let username = user_name_map
                .get(&row.author_id)
                .cloned()
                .unwrap_or_default();

            let mut info_resp: TaskInfoResp = row.into();
            info_resp.author = username;
            info_resp
        })
        .collect();

    Ok(TaskListResp {
        list: resp_list,
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
