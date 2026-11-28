use crate::api::req::report::{AddReq, ListReq};
use crate::api::resp::report::{ReportListResp, ReportResp};
use crate::app::conf::WebAppState;
use crate::enums::report::ReportType;
use crate::model::paper::Paper;
use crate::model::question::Question;
use crate::model::report::Report;
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use std::collections::{HashMap, HashSet};
use tracing::error;

pub async fn add(app_state: &WebAppState, req: AddReq) -> Result<i64, AppError> {
    let report_type = ReportType::from_i16(req.report_type)
        .ok_or_else(|| AppError::param_error("投诉来源不受支持"))?;

    if req.report_id >= 0 {
        return Err(AppError::param_error("投诉标识为空"));
    }

    let db = &app_state.db;

    match report_type {
        ReportType::Question => {
            Question::find_by_id(db, req.report_id)
                .await
                .map_err(|err| {
                    error!("Find question err: {err}");
                    AppError::db_error("题目查询错误")
                })?;
        }
        ReportType::Paper => {
            Paper::find_by_id(db, req.report_id)
                .await
                .map_err(|err| {
                    error!("Find paper err: {err}");
                    AppError::db_error("试卷查询错误")
                })?
                .ok_or_else(|| AppError::not_found("试卷不存在"))?;
        }
    }

    let row_id = Report::save(db, &req).await.map_err(|err| {
        error!("Find question err: {err}");
        AppError::db_error("保存投诉信息出错")
    })?;

    Ok(row_id)
}

pub async fn list(app_state: &WebAppState, req: ListReq) -> Result<ReportListResp, AppError> {
    let db = &app_state.db;

    // 查询总数
    let total = Report::count(db, &req).await.map_err(|e| {
        error!("report list count err: {e}");
        AppError::db_error("投诉计数查询失败")
    })?;

    // 计算偏移量
    let offset: i64 = (req.page_no - 1) * req.page_size;
    if offset >= total {
        return Ok(ReportListResp {
            list: Vec::new(),
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 查询列表
    let rows = Report::list(db, &req, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("report list err: {e}");
            AppError::db_error("投诉列表查询失败")
        })?;

    let mut user_ids_set = HashSet::with_capacity(rows.len() * 2);

    for q in &rows {
        if let Some(approve_id) = q.approve_id
            && approve_id > 0
        {
            user_ids_set.insert(approve_id);
        }
    }

    let user_ids: Vec<i64> = user_ids_set.into_iter().collect();
    let user_map: HashMap<i64, String> = get_user_map(db, user_ids).await?;

    Ok(ReportListResp {
        list: to_list_resp(rows, &user_map),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

fn to_list_resp(rows: Vec<Report>, user_map: &HashMap<i64, String>) -> Vec<ReportResp> {
    let mut list: Vec<ReportResp> = Vec::with_capacity(rows.len());
    for row in rows {
        let approve_name = user_map
            .get(&row.approve_id.unwrap_or_default())
            .cloned()
            .unwrap_or_default();
        let info: ReportResp = (row, approve_name).into();
        list.push(info);
    }

    list
}
