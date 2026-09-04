use crate::api::req::test::{LatestAttemptReq, ListReq, TestAnswerAddReq};
use crate::api::resp::paper::CommonPaperResp;
use crate::api::resp::test::{AttemptInfoResp, InfoResp, ListResp};
use crate::app::conf::AppState;
use crate::enums::test::{TestMethod, TestResult, TestStatus};
use crate::middleware::user::StudentUserInfo;
use crate::model::homework_class::HomeworkClass;
use crate::model::homework_class_student::HomeworkClassStudent;
use crate::model::homework_student_test_answer::HomeworkStudentTestAnswer;
use crate::model::homework_student_test_attempt::HomeworkStudentTestAttempt;
use crate::model::paper::Paper;
use crate::service::class_student::get_student_by_user_id;
use crate::util::error::AppError;
use crate::util::local::to_local_date;
use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info};

pub async fn list(
    app_state: &AppState,
    req: ListReq,
    user_info: StudentUserInfo,
) -> Result<ListResp, AppError> {
    if req.start_date.is_empty() || req.end_date.is_empty() {
        return Err(AppError::param_error("起止日期不能为空"));
    }

    let db = &app_state.db;

    let student = get_student_by_user_id(db, user_info.0.user_id).await?;

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
        req.page_size,
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
            AppError::db_error("获取学生作业布置信息出错")
        })?;
    // 作业标识id-> 作业信息
    let homework_id_class_map: HashMap<i64, &HomeworkClass> = homework_rows
        .iter()
        .map(|item| (item.homework_id, item))
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
        let (paper_id, deadline) = if let Some(hc) = homework_id_class_map.get(&item.homework_id) {
            (hc.paper_id, to_local_date(hc.deadline))
        } else {
            error!("test list homework id is empty: {}", item.homework_id);
            (0, "".to_string())
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
            homework_id: item.homework_id,
            student_id: item.student_id,
            deadline,
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

pub async fn attempt_latest(
    app_state: &AppState,
    req: LatestAttemptReq,
    user_info: StudentUserInfo,
) -> Result<AttemptInfoResp, AppError> {
    let test_method =
        TestMethod::from_i16(req.method).ok_or_else(|| AppError::param_error("做题模式不支持"))?;

    let db = &app_state.db;

    // 学生信息
    let student = get_student_by_user_id(db, user_info.0.user_id).await?;

    // 学生作业布置信息
    let hcs = HomeworkClassStudent::find_by_id(db, req.id)
        .await
        .map_err(|e| {
            error!("get test latest attempt row error: {}", e);
            AppError::db_error("获取学生作业布置信息出错")
        })?
        .ok_or_else(|| AppError::not_found("作业布置信息不存在"))?;

    if hcs.student_id != student.id {
        return Err(AppError::business_error("这不是你的作业"));
    }

    // 作业详情
    let hc_rows = HomeworkClass::find_by_homework_ids(db, vec![hcs.homework_id])
        .await
        .map_err(|e| {
            error!("get test latest homework class rows error: {}", e);
            AppError::db_error("获取班级作业信息失败")
        })?;

    if hc_rows.is_empty() {
        return Err(AppError::not_found("作业布置信息为空"));
    }
    if hc_rows.len() > 1 {
        return Err(AppError::business_error("作业布置不匹配"));
    }
    let hc = hc_rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::not_found("作业布置信息不存在"))?;

    // 先尝试获取当前最新的做题记录
    let maybe_hsta = HomeworkStudentTestAttempt::find_in_progress_latest_attempt(
        db,
        hcs.homework_id,
        hcs.student_id,
        req.method,
    )
    .await
    .map_err(|e| {
        error!("get test latest in progress attempt row error: {}", e);
        AppError::db_error("查询最新的作业记录失败")
    })?;

    // 使用 match 平铺处理异步逻辑 避免闭包编译报错
    let mut hsta = match maybe_hsta {
        // 如果能查到记录，且该记录还没写完（比如你可能需要在外面判断 status == InProgress，或者这个方法本身就只查进行中的数据）
        Some(record) => record,

        // 如果没有记录 或者已有记录都已完成 需要开启新一轮
        None => {
            let max_no = HomeworkStudentTestAttempt::find_max_attempt_number(
                db,
                hcs.homework_id,
                hcs.student_id,
            )
            .await
            .map_err(|e| {
                error!("test latest max attempt number error: {}", e);
                AppError::db_error("获取做题记录批次失败")
            })?
            .unwrap_or(0);

            // 开启新一轮做题，批次号在历史最大值基础上 + 1
            HomeworkStudentTestAttempt {
                id: None,
                student_id: hcs.student_id,
                homework_id: hcs.homework_id,
                class_id: hc.class_id,
                paper_id: hc.paper_id,
                attempt_number: max_no + 1,
                method: test_method.as_i16(),
                status: TestStatus::InProgress.as_i16(),
                score: None,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                completed_at: None,
            }
        }
    };

    // 3. 如果是新生成的批次数据（id 为 None），将其落库保存
    if hsta.id.is_none() {
        let id = HomeworkStudentTestAttempt::save(db, &hsta)
            .await
            .map_err(|e| {
                error!("save test latest attempt row error: {}", e);
                AppError::db_error("记录最新的做题记录失败")
            })?;
        hsta.id = Some(id);
    }

    // 获取做题明细记录
    let answers = HomeworkStudentTestAnswer::find_by_attempt_id(db, hsta.id.unwrap_or_default())
        .await
        .map_err(|e| {
            error!("get test latest answer row error: {}", e);
            AppError::db_error("获取答案明细出错")
        })?;

    let mut resp: AttemptInfoResp = hsta.into();
    resp.answers = answers.into_iter().map(Into::into).collect();

    Ok(resp)
}

async fn validate_attempt(db: &PgPool, user_id: i64, attempt_id: i64) -> Result<(), AppError> {
    // 学生信息
    let student = get_student_by_user_id(db, user_id).await?;

    // 做题记录
    let attempt = HomeworkStudentTestAttempt::find_by_id(db, attempt_id)
        .await
        .map_err(|e| {
            error!("find test attempt row error: {}", e);
            AppError::db_error("做题记录查询失败")
        })?
        .ok_or_else(|| AppError::not_found("做题记录为空"))?;
    if student.id != attempt.student_id {
        return Err(AppError::business_error("作业信息有错误"));
    }

    Ok(())
}

pub async fn answer_add(
    app_state: &AppState,
    req: TestAnswerAddReq,
    user_info: StudentUserInfo,
) -> Result<bool, AppError> {
    let test_status = TestStatus::from_i16(req.status)
        .ok_or_else(|| AppError::param_error("做题记录状态错误"))?;

    if req.list.is_empty() {
        return Err(AppError::param_error("你选择的答案为空"));
    }

    let a_id = req.attempt_id;

    let db = &app_state.db;

    validate_attempt(db, user_info.0.user_id, req.attempt_id).await?;

    let add_list = build_add_req(req)?;

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to answer add begin transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    // 删除历史答案
    let rows = HomeworkStudentTestAnswer::delete_by_attempt_id(&mut tx, a_id)
        .await
        .map_err(|e| {
            error!("delete by attempt id error: {}", e);
            AppError::db_error("删除做题记录失败")
        })?;
    info!("delete attempt id: {}, rows: {}", a_id, rows);

    // 重新写入新答案
    let id = HomeworkStudentTestAnswer::batch_insert(&mut tx, &add_list)
        .await
        .map_err(|e| {
            error!("save test answer add error: {}", e);
            AppError::db_error("答案保存失败")
        })?;

    // 得分暂时计算意义不大, 因此已交卷需要更新尝试做题记录表
    if test_status == TestStatus::Done {
        let row = HomeworkStudentTestAttempt::done_by_id(&mut tx, a_id)
            .await
            .map_err(|e| {
                error!("save test attempt add error: {}", e);
                AppError::db_error("更新做题记录完成状态失败")
            })?;
        info!("save test attempt id: {}, row: {}", a_id, row);
    }
    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to answer add commit transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    Ok(id > 0)
}

fn build_add_req(req_list: TestAnswerAddReq) -> Result<Vec<HomeworkStudentTestAnswer>, AppError> {
    let mut add_list: Vec<HomeworkStudentTestAnswer> = vec![];
    for req in req_list.list.into_iter() {
        let test_result = TestResult::from_i16(req.result)
            .ok_or_else(|| AppError::param_error("答案正确与否处理错误"))?;
        add_list.push(HomeworkStudentTestAnswer {
            id: None,
            attempt_id: req_list.attempt_id,
            question_id: req.question_id,
            answer: req.answer,
            result: test_result.as_i16(),
            note: req.note,
            remark: "".to_string(),
            created_at: None,
            updated_at: None,
        })
    }

    Ok(add_list)
}
