use crate::api::req::test::ListReq;
use crate::api::resp::paper::CommonPaperResp;
use crate::api::resp::test::{InfoResp, ListResp};
use crate::app::conf::AppState;
use crate::middleware::user::StudentUserInfo;
use crate::model::class_student::ClassStudent;
use crate::model::homework_class::HomeworkClass;
use crate::model::homework_class_student::HomeworkClassStudent;
use crate::model::paper::Paper;
use crate::util::error::AppError;
use std::collections::HashMap;
use tracing::error;

pub async fn list(
    app_state: &AppState,
    req: ListReq,
    user_info: StudentUserInfo,
) -> Result<ListResp, AppError> {
    let db = &app_state.db;

    let student = ClassStudent::find_by_user_id(db, user_info.0.user_id)
        .await
        .map_err(|e| {
            error!("find_by_user_id error: {}", e);
            AppError::db_error("获取学生信息出错")
        })?
        .ok_or_else(|| {
            error!("find by user id: {} is empty", user_info.0.user_id);
            AppError::business_error("学生账户不存在")
        })?;

    let total = HomeworkClassStudent::count(db, student.id, &req.start_date, &req.end_date)
        .await
        .map_err(|e| {
            error!("test list count error: {}", e);
            AppError::db_error("查询学生任务数量出错")
        })?;

    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(ListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    let rows = HomeworkClassStudent::list(
        db,
        student.id,
        &req.start_date,
        &req.end_date,
        req.page_no,
        offset,
    )
    .await
    .map_err(|e| {
        error!("test list rows error: {}", e);
        AppError::db_error("查询学生任务列表出错")
    })?;

    // 查询所有的作业信息
    let mut homework_ids: Vec<i64> = rows.iter().map(|row| row.homework_id).collect();
    homework_ids.sort_unstable();
    homework_ids.dedup();
    let homework_rows = HomeworkClass::find_by_homework_ids(db, homework_ids)
        .await
        .map_err(|e| {
            error!("test list homework rows error: {}", e);
            AppError::db_error("获取学生的作业布置信息戳错")
        })?;
    // 作业标识id->试卷id
    let homework_id_paper_id_map: HashMap<i64, i64> = homework_rows
        .iter()
        .map(|item| (item.homework_id, item.paper_id))
        .collect();

    // 获取试卷信息
    let mut paper_ids: Vec<i64> = homework_rows.iter().map(|row| row.paper_id).collect();
    paper_ids.sort_unstable();
    paper_ids.dedup();
    let paper_list = Paper::find_by_ids(db, paper_ids).await.map_err(|e| {
        error!("test list paper rows error: {}", e);
        AppError::db_error("获取学生作业试卷信息出错")
    })?;
    // 试卷id->试卷详情
    let paper_map: HashMap<i64, &Paper> = paper_list
        .iter()
        .map(|item| (item.id.unwrap_or_default(), item))
        .collect();

    // 试卷完成情况信息记录

    let mut resp: Vec<InfoResp> = Vec::new();
    for item in rows.into_iter() {
        // 获取对应的试卷标识
        let paper_id = if let Some(paper_id) = homework_id_paper_id_map.get(&item.homework_id) {
            paper_id
        } else {
            error!("test list homework id is empty: {}", item.homework_id);
            &0
        };

        // 获取试卷详情, 试卷不存在默认空
        let paper_resp = if let Some(paper) = paper_map.get(&paper_id) {
            (**paper).clone().into()
        } else {
            error!("test list paper info is empty: {}", paper_id);
            CommonPaperResp::default()
        };

        resp.push(InfoResp {
            id: item.id,
            deadline: "2026-09-02".to_string(),
            paper_info: paper_resp,
        })
    }

    Ok(ListResp {
        list: resp,
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
