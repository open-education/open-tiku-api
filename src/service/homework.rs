use crate::api::resp::class::ClassInfoResp;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::model::class::Class;
use crate::model::class_student::ClassStudent;
use crate::model::homework_class::HomeworkClass;
use crate::model::homework_class_student::HomeworkClassStudent;

use crate::api::req::homework::{HomeworkAddReq, HomeworkListReq};
use crate::api::resp::class_student::ClassStudentResp;
use crate::api::resp::homework::{HomeworkInfoResp, HomeworkListResp};
use crate::service::class_student::check_class_list_info;
use crate::util::error::AppError;
use crate::util::local::to_local_datetime;
use crate::util::snowflake::generate_id;
use std::collections::HashMap;
use tracing::{error, info};

// 布置作业

// 获取批次号
pub async fn batch_no(app_state: &AppState, paper_id: i64) -> Result<i32, AppError> {
    let max_batch_no = HomeworkClass::get_max_batch_no(&app_state.db, paper_id)
        .await
        .map_err(|err| {
            error!("Get paper_id: {} batch no err: {}", paper_id, err);
            AppError::db_error("获取试卷批次号错误")
        })?
        .unwrap_or(0);

    Ok(max_batch_no + 1)
}

pub async fn add(
    app_state: &AppState,
    req: HomeworkAddReq,
    teacher_user_info: TeacherUserInfo,
) -> Result<bool, AppError> {
    if req.title.is_empty() {
        return Err(AppError::param_error("标题不能为空"));
    }
    if req.class_map.is_empty() {
        return Err(AppError::param_error("班级信息为空"));
    }

    // 批次号要匹配, 避免同一批次好重复添加, 如果冲突太多, 后续可以考虑优化, 获取不加入判断, 自动自增
    let cur_max_batch_no = batch_no(&app_state, req.paper_id).await?;
    if cur_max_batch_no != req.batch_no {
        return Err(AppError::param_error(
            "当前批次号已被其它教师使用，需要刷新后重新布置作业",
        ));
    }

    let db = &app_state.db;

    // 校验班级信息是否合法
    let class_ids: Vec<i64> = req.class_map.keys().copied().collect();
    check_class_list_info(db, class_ids, teacher_user_info.0.user_id, true).await?;

    // 生成批量入库信息, homework_id 为提前生成
    let (class_list, class_students) = build_homework_add_req(req, teacher_user_info.0.user_id)?;

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to homework add begin transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    let class_rows = HomeworkClass::batch_insert(&mut tx, &class_list)
        .await
        .map_err(|e| {
            error!("Failed to add homework class to transaction: {}", e);
            AppError::db_error("布置作业写入班级信息失败")
        })?;
    info!(
        "Added homework class to transaction class_rows: {}",
        class_rows
    );

    let class_students_rows = HomeworkClassStudent::batch_insert(&mut tx, &class_students)
        .await
        .map_err(|e| {
            error!("Failed to add homework class to transaction: {}", e);
            AppError::db_error("布置作业写入班级学生信息失败")
        })?;
    info!(
        "Added homework class to transaction class_students_rows: {}",
        class_students_rows
    );

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to homework add commit transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    Ok(class_rows > 0 && class_students_rows > 0)
}

fn build_homework_add_req(
    req: HomeworkAddReq,
    author_id: i64,
) -> Result<(Vec<HomeworkClass>, Vec<HomeworkClassStudent>), AppError> {
    let mut class_list: Vec<HomeworkClass> = vec![];
    let mut class_students: Vec<HomeworkClassStudent> = vec![];
    for (class_id, student_ids) in req.class_map.iter() {
        // 班级信息不能为空
        if student_ids.is_empty() {
            return Err(AppError::param_error("班级学生账户为空"));
        }

        // 生成作业标识
        let homework_id = generate_id();
        for student_id in student_ids {
            class_students.push(HomeworkClassStudent {
                id: 0,
                homework_id,
                student_id: *student_id,
            })
        }

        class_list.push(HomeworkClass {
            id: None,
            batch_no: req.batch_no,
            homework_id,
            paper_id: req.paper_id,
            class_id: *class_id,
            author_id,
            title: req.title.clone(),
            remark: req.remark.clone().unwrap_or_default(),
            created_at: None,
        })
    }

    Ok((class_list, class_students))
}

pub async fn list(
    app_state: &AppState,
    req: HomeworkListReq,
    teacher_user_info: TeacherUserInfo,
) -> Result<HomeworkListResp, AppError> {
    let db = &app_state.db;
    let user_id = teacher_user_info.0.user_id;

    // 获取总数
    let total = HomeworkClass::count(db, user_id, req.paper_id, req.batch_no)
        .await
        .map_err(|e| {
            error!("Count homework class error: {}", e);
            AppError::db_error("班级作业布置计数查询错误")
        })?;

    // 分页边界检查
    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 || total == 0 {
        return Ok(HomeworkListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 查询作业列表
    let rows = HomeworkClass::list(
        db,
        user_id,
        req.paper_id,
        req.batch_no,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("List homework class error: {}", e);
        AppError::db_error("班级作业布置列表查询失败")
    })?;

    if rows.is_empty() {
        return Ok(HomeworkListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 获取班级信息
    let mut class_ids: Vec<i64> = rows.iter().map(|row| row.class_id).collect();
    class_ids.sort_unstable();
    class_ids.dedup();
    let classes = Class::find_by_ids(db, class_ids).await.map_err(|e| {
        error!("List homework class error: {}", e);
        AppError::db_error("获取班级信息失败")
    })?;
    let class_map: HashMap<i64, &Class> = classes
        .iter()
        .map(|item| (item.id.unwrap_or_default(), item))
        .collect();

    // 获取作业关联的学生映射
    let mut homework_ids: Vec<i64> = rows.iter().map(|row| row.homework_id).collect();
    homework_ids.sort_unstable();
    homework_ids.dedup();

    // 获取作业关联的学生列表
    let students = HomeworkClassStudent::find_by_homework_ids(db, homework_ids)
        .await
        .map_err(|e| {
            error!("List homework class students error: {}", e);
            AppError::db_error("班级作业布置学生列表查询错误")
        })?;

    let mut student_map: HashMap<i64, Vec<&HomeworkClassStudent>> =
        HashMap::with_capacity(rows.len());
    for student in &students {
        student_map
            .entry(student.homework_id)
            .or_default()
            .push(student);
    }

    // 获取学生账户映射
    let mut student_ids: Vec<i64> = students.iter().map(|student| student.student_id).collect();
    student_ids.sort_unstable();
    student_ids.dedup();

    // 获取学生账户信息
    let accounts = ClassStudent::find_by_ids(db, student_ids)
        .await
        .map_err(|e| {
            error!("List class student error: {}", e);
            AppError::db_error("学生账户信息列表查询失败")
        })?;

    let account_map: HashMap<i64, &ClassStudent> = accounts
        .iter()
        .map(|account| (account.id, account))
        .collect();

    // 组装返回结果
    let mut resp: Vec<HomeworkInfoResp> = Vec::with_capacity(rows.len());
    for item in rows {
        // 班级信息
        let class_info = if let Some(class_row) = class_map.get(&item.class_id) {
            (**class_row).clone().into()
        } else {
            error!("Homework class id not found: {}", item.class_id);
            ClassInfoResp::default()
        };

        // 学生账户信息
        let mut account_list: Vec<ClassStudentResp> = vec![];

        if let Some(student_list) = student_map.remove(&item.homework_id) {
            for info in student_list {
                if let Some(account_info) = account_map.get(&info.student_id) {
                    account_list.push((*account_info).clone().into());
                } else {
                    error!("Homework class student_id not found: {}", info.student_id);
                }
            }
        } else {
            error!(
                "Homework class student homework_id not found: {}",
                item.homework_id
            );
        }

        resp.push(HomeworkInfoResp {
            id: item.id.unwrap_or_default(),
            batch_no: item.batch_no,
            homework_id: item.homework_id,
            paper_id: item.paper_id,
            class_id: item.class_id,
            class_info,
            author_id: item.author_id,
            title: item.title,
            remark: item.remark,
            students: account_list,
            created_at: if let Some(created_at) = item.created_at {
                to_local_datetime(created_at)
            } else {
                "".to_string()
            },
        });
    }

    Ok(HomeworkListResp {
        list: resp,
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
