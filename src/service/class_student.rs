use crate::api::class_student::{
    ClassStudentEditReq, ClassStudentListReq, ClassStudentReq, ClassStudentResp,
};
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::model::class::Class;
use crate::model::class_student::{ClassStudent, StudentStatus};
use crate::util::argon2::{generate_random_password, hash_password};
use crate::util::email::{get_student_account_html, send_html_email};
use crate::util::error::AppError;
use crate::util::local::to_local_datetime;
use crate::util::snowflake;
use chrono::Utc;
use futures_util::future::try_join_all;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::timeout;
use tracing::{error, info};

// 添加学生账户
pub async fn add(
    app_state: &AppState,
    req: ClassStudentReq,
    user_info: TeacherUserInfo,
) -> Result<u64, AppError> {
    check_student_add_req(&req)?;

    let account_set = get_account_set(req.accounts);
    let accounts: Vec<String> = account_set.into_iter().collect();
    if accounts.is_empty() {
        return Err(AppError::param_error("没有有效的学生账户"));
    }

    let db = &app_state.db;

    let class_row = check_class_info(db, req.class_id, user_info.0.user_id, true).await?;

    // 验证账户是否存在-登录账户必须是全局的唯一
    check_student_accounts(db, &accounts).await?;

    // 如果是全量导入则删除该班级已有的账户列表
    if !req.incremental {
        // 其实如果账户属于班级内可以覆盖, 暂时没写这个逻辑
        let del_rows = ClassStudent::delete_by_class_id(db, req.class_id)
            .await
            .map_err(|err| {
                error!(
                    "Add class student delete by class id {} err: {}",
                    req.class_id, err
                );
                AppError::db_error("全量导入时清空班级已有账户失败")
            })?;
        info!(
            "Delete class id: {} student rows: {}",
            req.class_id, del_rows
        );
    }

    // 记录账户和登录密码
    let (add_list, account_to_map) = build_student_req(
        app_state.config.login.student_pepper.clone(),
        req.class_id,
        accounts,
    )
    .await?;

    let count = ClassStudent::batch_insert(db, &add_list)
        .await
        .map_err(|e| {
            error!("Batch insert err: {}", e);
            AppError::db_error("导入班级学生出错")
        })?;

    send_account_email(app_state, &class_row, account_to_map).await?;

    Ok(count)
}

// 检查必填参数
fn check_student_add_req(req: &ClassStudentReq) -> Result<(), AppError> {
    if req.class_id <= 0 {
        return Err(AppError::param_error("班级为空"));
    }
    if req.accounts.is_empty() {
        return Err(AppError::param_error("账户列表为空"));
    }

    Ok(())
}

// 去除账户两边的空格等特殊字符并去重
fn get_account_set(input: String) -> HashSet<String> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

// 检查班级信息
async fn check_class_info(
    db: &PgPool,
    class_id: i64,
    user_id: i64,
    check_email: bool,
) -> Result<Class, AppError> {
    let class_list = check_class_list_info(db, vec![class_id], user_id, check_email).await?;

    let class = class_list
        .into_iter()
        .next()
        .ok_or_else(|| AppError::not_found("班级信息不存在"))?;

    Ok(class)
}

// 检查班级列表信息
pub async fn check_class_list_info(
    db: &PgPool,
    class_ids: Vec<i64>,
    user_id: i64,
    check_email: bool,
) -> Result<Vec<Class>, AppError> {
    let class_row_list = Class::find_by_ids(db, class_ids).await.map_err(|err| {
        error!("Select class err: {}", err);
        AppError::db_error("班级查询错误")
    })?;
    if class_row_list.is_empty() {
        return Err(AppError::param_error("班级为空"));
    }

    for class_row in class_row_list.iter() {
        if class_row.author_id != user_id {
            return Err(AppError::permission_denied("你只能管理自己的班级"));
        }
        if check_email && class_row.email.is_empty() {
            return Err(AppError::business_error(
                "没有配置个人邮箱, 无法接收账户登录密码",
            ));
        }
    }

    Ok(class_row_list)
}

// 验证学生账户是否存在
async fn check_student_accounts(db: &PgPool, accounts: &Vec<String>) -> Result<(), AppError> {
    // 验证账户是否存在-登录账户必须是全局的唯一
    let has_rows = ClassStudent::find_by_accounts(db, &accounts)
        .await
        .map_err(|e| {
            error!("Select class student by account err: {}", e);
            AppError::db_error("班级学生查询错误")
        })?;
    let has_accounts: Vec<String> = has_rows.into_iter().map(|item| item.account).collect();
    if !has_accounts.is_empty() {
        return Err(AppError::business_error(
            format!("以下账户: {} 已存在, 无法增量导入", has_accounts.join(", ")).as_str(),
        ));
    }

    Ok(())
}

// 构建班级学生账户
async fn build_student_req(
    pepper: String,
    class_id: i64,
    accounts: Vec<String>,
) -> Result<(Vec<ClassStudent>, HashMap<String, String>), AppError> {
    // 控制并发数
    let semaphore = Arc::new(Semaphore::new(5));

    // 为每个账户创建异步任务 但内部用 spawn_blocking 执行哈希
    let tasks: Vec<_> = accounts
        .into_iter()
        .map(|account| {
            let pepper = pepper.clone();
            let permit = semaphore.clone().acquire_owned(); // 异步获取许可
            task::spawn_blocking(move || {
                // 此处为阻塞的哈希计算
                let _permit = permit; // 持有许可直到任务结束
                let password = generate_random_password();
                let hashed = hash_password(&pepper, &password).map_err(|err| {
                    error!("Generate student account {} password err: {}", account, err);
                    AppError::internal_error("生成学生密码失败, 请重试")
                })?;
                let student = ClassStudent {
                    id: 0,
                    class_id,
                    user_id: snowflake::generate_id(),
                    account: account.clone(),
                    password: hashed,
                    status: StudentStatus::Active.as_i16(),
                    remark: "".to_string(),
                    last_login_time: None,
                    login_count: 0,
                    created_at: None,
                    updated_at: None,
                };
                Ok((account, password, student))
            })
        })
        .collect();

    // 等待所有任务完成 任一出错即中断, 并施加超时 超时 120秒
    let results = timeout(Duration::from_secs(120), try_join_all(tasks))
        .await
        .map_err(|_| AppError::internal_error("导入超时，请稍后重试"))?
        .map_err(|join_err| {
            AppError::internal_error(format!("并行任务失败: {}", join_err).as_str())
        })?
        .into_iter()
        .collect::<Result<Vec<_>, AppError>>()?;

    // 构建返回数据
    let mut rows = Vec::with_capacity(results.len());
    let mut account_to_pwd = HashMap::with_capacity(results.len());
    for (account, password, student) in results {
        account_to_pwd.insert(account, password);
        rows.push(student);
    }
    Ok((rows, account_to_pwd))
}

// 发送学生账户密码文件给教师个人邮箱
async fn send_account_email(
    app_state: &AppState,
    class_info: &Class,
    account_to_map: HashMap<String, String>,
) -> Result<(), AppError> {
    let account_htm = get_student_account_html(&account_to_map);

    // 邮件标题
    let mut titles: Vec<String> = Vec::new();
    titles.push(to_local_datetime(Utc::now()));
    titles.push(class_info.year.to_string());
    if !class_info.grade.is_empty() {
        titles.push(class_info.grade.to_string());
    }
    if !class_info.semester.is_empty() {
        titles.push(class_info.semester.to_string());
    }
    titles.push(class_info.label.to_string());
    titles.push("学生账户相关信息".to_string());

    let title = titles.join("-");

    send_html_email(
        &app_state.config.smtp,
        class_info.email.as_str(),
        title.as_str(),
        account_htm.as_str(),
    )
    .await?;

    info!("email title: {} send success", title);

    Ok(())
}

// 班级内学生账户列表
pub async fn list(
    app_state: &AppState,
    req: ClassStudentListReq,
    user_info: TeacherUserInfo,
) -> Result<HashMap<i64, Vec<ClassStudentResp>>, AppError> {
    if req.class_ids.is_empty() {
        return Err(AppError::param_error("班级请求信息为空"));
    }

    let db = &app_state.db;

    check_class_list_info(db, req.class_ids.clone(), user_info.0.user_id, false).await?;

    let rows = ClassStudent::find_by_class_ids(db, req.class_ids)
        .await
        .map_err(|err| {
            error!("Select class err: {}", err);
            AppError::db_error("班级账号查询错误")
        })?;

    let mut map: HashMap<i64, Vec<ClassStudent>> = HashMap::new();
    for student in rows {
        map.entry(student.class_id)
            .or_insert_with(Vec::new)
            .push(student);
    }

    let resp_map: HashMap<_, _> = map
        .into_iter()
        .map(|(class_id, students)| {
            let converted = students.into_iter().map(to_info_resp).collect();
            (class_id, converted)
        })
        .collect();

    Ok(resp_map)
}

pub fn to_info_resp(raw: ClassStudent) -> ClassStudentResp {
    ClassStudentResp {
        id: raw.id,
        class_id: raw.class_id,
        user_id: raw.user_id,
        account: raw.account,
        status: raw.status,
        status_desc: StudentStatus::desc(raw.status),
        remark: raw.remark,
        last_login_time: if raw.last_login_time.is_none() {
            "".to_string()
        } else {
            to_local_datetime(raw.last_login_time.unwrap_or_default())
        },
        login_count: raw.login_count,
        created_at: to_local_datetime(raw.created_at.unwrap_or_default()),
        updated_at: to_local_datetime(raw.updated_at.unwrap_or_default()),
    }
}

// 编辑用户信息
pub async fn edit(
    app_state: &AppState,
    req: ClassStudentEditReq,
    user_info: TeacherUserInfo,
) -> Result<bool, AppError> {
    validate_student_edit_req(&req)?;

    let account = req.account.clone().trim().to_string();

    let db = &app_state.db;

    let class_row = check_class_info(db, req.class_id, user_info.0.user_id, true).await?;

    let student = ClassStudent::find_by_id(db, req.id)
        .await
        .map_err(|err| {
            error!("Select class student {} err: {}", req.id, err);
            AppError::db_error("查询学生账户信息错误")
        })?
        .ok_or_else(|| AppError::not_found("学生账户不存在"))?;

    // 检查学生账户是否可编辑
    check_student_is_edit(db, &account, &student).await?;

    let mut edit_req: ClassStudent = ClassStudent {
        id: student.id,
        class_id: req.class_id,
        user_id: student.user_id,
        account: account.clone(),
        password: student.password.clone(),
        status: StudentStatus::from_i16(req.status).as_i16(),
        remark: req.remark,
        last_login_time: student.last_login_time,
        login_count: student.login_count,
        created_at: None,
        updated_at: None,
    };

    // 生成密码
    let mut password: String = "".to_string();
    if req.reset_pwd {
        password = generate_random_password();
        let hashed =
            hash_password(&app_state.config.login.student_pepper, &password).map_err(|err| {
                error!(
                    "Generate student account {} password err: {}",
                    req.account, err
                );
                AppError::internal_error("生成学生密码失败, 请重试")
            })?;

        // 更细密码
        edit_req.password = hashed;
    }

    // 更新账户信息
    let rows = ClassStudent::update_by_id(db, &edit_req)
        .await
        .map_err(|err| {
            error!("Update student account {} err: {}", req.id, err);
            AppError::db_error("更新学生账户信息失败")
        })?;

    // 如果重置密码则发送通知邮件
    if req.reset_pwd {
        let mut account_to_map: HashMap<String, String> = HashMap::new();
        account_to_map
            .entry(req.account.clone())
            .or_insert(password);
        send_account_email(app_state, &class_row, account_to_map).await?;
    }

    Ok(rows > 0)
}

// 检查必填参数
fn validate_student_edit_req(req: &ClassStudentEditReq) -> Result<(), AppError> {
    if req.class_id <= 0 {
        return Err(AppError::param_error("班级为空"));
    }
    if req.account.is_empty() {
        return Err(AppError::param_error("账户列表为空"));
    }

    Ok(())
}

// 验证学生账户是否可以编辑
async fn check_student_is_edit(
    db: &PgPool,
    account: &str,
    student: &ClassStudent,
) -> Result<(), AppError> {
    // 验证账户是否存在-登录账户必须是全局的唯一
    let has_rows = ClassStudent::find_by_account(db, account)
        .await
        .map_err(|e| {
            error!("Select class student {} err: {}", account, e);
            AppError::db_error("班级学生查询错误")
        })?;
    // 不存在说明是新用户名称
    if has_rows.is_none() {
        return Ok(());
    }

    // 或者该账户是当前用户也可以修改
    if let Some(row) = has_rows {
        if row.id == student.id {
            return Ok(());
        }
    }

    Err(AppError::business_error(
        format!("账户: {} 已存在, 无法修改", account).as_str(),
    ))
}

// 通过用户获取学生信息, 不存在返回错误
pub async fn get_student_by_user_id(db: &PgPool, user_id: i64) -> Result<ClassStudent, AppError> {
    let student = ClassStudent::find_by_user_id(db, user_id)
        .await
        .map_err(|e| {
            error!("Select class student {} err: {}", user_id, e);
            AppError::db_error("学生账户查询出错")
        })?
        .ok_or_else(|| AppError::not_found("学生账户不存在"))?;

    Ok(student)
}
