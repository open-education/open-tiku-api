use crate::api::homework::HomeworkAddReq;
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::model::homework_class::HomeworkClass;
use crate::model::homework_class_student::HomeworkClassStudent;
use crate::service::class_student::check_class_list_info;
use crate::util::error::AppError;
use crate::util::snowflake::generate_id;
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
        .unwrap_or(1);

    Ok(max_batch_no)
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
                homework_id,
                student_id: *student_id,
            })
        }

        class_list.push(HomeworkClass {
            batch_no: req.batch_no,
            homework_id,
            paper_id: req.paper_id,
            class_id: *class_id,
            author_id,
            title: req.title.clone(),
            remark: req.remark.clone().unwrap_or_default(),
        })
    }

    Ok((class_list, class_students))
}
