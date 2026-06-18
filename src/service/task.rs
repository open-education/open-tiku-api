use crate::AppConfig;
use crate::api::task::{TaskAddReq, TaskInfoResp, TaskListReq, TaskListResp};
use crate::model::task::{Task, TaskStatus};
use crate::util::local::to_local_datetime;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 添加任务
pub async fn add(app_conf: web::Data<AppConfig>, req: TaskAddReq) -> Result<i64, Error> {
    let db = &app_conf.get_ref().db;

    let row_id = Task::insert(
        db,
        req.question_cate_id,
        req.task_type,
        &req.name,
        1,
        &req.url,
        &req.email,
        req.textbook_id,
    )
    .await
    .map_err(|e| {
        error!("task add err: {:?}", e);
        Error::new(ErrorKind::Other, "任务添加失败")
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

pub async fn list(app_conf: web::Data<AppConfig>, req: TaskListReq) -> Result<TaskListResp, Error> {
    let db = &app_conf.db;

    // 1. 查询总数
    let total = Task::count_by_cate(db, req.question_cate_id, 1, req.task_type)
        .await
        .map_err(|e| {
            error!("task count by id err: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
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
        Error::new(ErrorKind::Other, "查询失败")
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
