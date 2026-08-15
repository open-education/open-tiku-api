use crate::AppConfig;
use crate::api::class_student::{ClassStudentEditReq, ClassStudentReq, ClassStudentResp};
use crate::middleware::user::TeacherUserInfo;
use crate::model::class::Class;
use crate::model::class_student::{ClassStudent, StudentStatus};
use crate::util::argon2::{generate_random_password, hash_password};
use crate::util::email::{EmailConfig, get_student_account_html, send_html_email};
use crate::util::local::to_local_datetime;
use actix_web::web;
use chrono::Utc;
use futures_util::future::try_join_all;
use log::{error, info};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time::timeout;

// 添加学生账户
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: ClassStudentReq,
    user_info: TeacherUserInfo,
) -> Result<u64, Error> {
    check_student_add_req(&req)?;

    let account_set = get_account_set(req.accounts);
    let accounts: Vec<String> = account_set.into_iter().collect();
    if accounts.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "没有有效的学生账户"));
    }

    let db = &app_conf.db;

    let class_row = check_class_info(db, req.class_id, user_info.0.user_id, true).await?;

    // 验证账户是否存在-登录账户必须是全局的唯一
    check_student_accounts(db, &accounts).await?;

    // 如果是全量导入则删除该班级已有的账户列表
    if !req.incremental {
        // 其实如果账户属于班级内可以覆盖, 暂时没写这个逻辑
        let del_rows = ClassStudent::delete_by_class_id(db, req.class_id)
            .await
            .map_err(|err| {
                Error::new(ErrorKind::Other, format!("{:?}", err));
                Error::new(ErrorKind::InvalidInput, "全量导入时清空班级已有账户失败")
            })?;
        info!(
            "Delete class id: {} student rows: {}",
            req.class_id, del_rows
        );
    }

    // 记录账户和登录密码
    let (add_list, account_to_map) =
        build_student_req(app_conf.student_pepper.clone(), req.class_id, accounts).await?;

    let count = ClassStudent::batch_insert(db, &add_list)
        .await
        .map_err(|e| {
            error!("Batch insert err: {}", e);
            Error::new(ErrorKind::InvalidInput, "导入班级学生出错")
        })?;

    send_account_email(app_conf, &class_row, account_to_map).await?;

    Ok(count)
}

// 检查必填参数
fn check_student_add_req(req: &ClassStudentReq) -> Result<(), Error> {
    if req.class_id <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "班级为空"));
    }
    if req.accounts.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "账户列表为空"));
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
) -> Result<Class, Error> {
    let class_row = Class::find_by_id(db, class_id)
        .await
        .map_err(|err| {
            error!("Select class err: {}", err);
            Error::new(ErrorKind::InvalidInput, "班级查询错误")
        })?
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "班级不存在"))?;
    if class_row.author_id != user_id {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "你只能管理自己的班级",
        ));
    }
    if check_email && class_row.email.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "没有配置个人邮箱, 无法接收账户登录密码",
        ));
    }

    Ok(class_row)
}

// 验证学生账户是否存在
async fn check_student_accounts(db: &PgPool, accounts: &Vec<String>) -> Result<(), Error> {
    // 验证账户是否存在-登录账户必须是全局的唯一
    let has_rows = ClassStudent::find_by_accounts(db, &accounts)
        .await
        .map_err(|e| {
            Error::new(ErrorKind::Other, format!("{:?}", e));
            Error::new(ErrorKind::InvalidInput, "班级学生查询错误")
        })?;
    let has_accounts: Vec<String> = has_rows.into_iter().map(|item| item.account).collect();
    if !has_accounts.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("以下账户: {} 已存在, 无法增量导入", has_accounts.join(", ")),
        ));
    }

    Ok(())
}

// 构建班级学生账户
async fn build_student_req(
    pepper: String,
    class_id: i64,
    accounts: Vec<String>,
) -> Result<(Vec<ClassStudent>, HashMap<String, String>), Error> {
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
                    Error::new(ErrorKind::InvalidInput, "生成学生密码失败, 请重试")
                })?;
                let student = ClassStudent {
                    id: 0,
                    class_id,
                    account: account.clone(),
                    password: hashed,
                    status: StudentStatus::Active.as_i16(),
                    remark: "".to_string(),
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
        .map_err(|_| Error::new(ErrorKind::TimedOut, "导入超时，请稍后重试"))?
        .map_err(|join_err| Error::new(ErrorKind::Other, format!("并行任务失败: {}", join_err)))?
        .into_iter()
        .collect::<Result<Vec<_>, Error>>()?;

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
    app_conf: web::Data<AppConfig>,
    class_info: &Class,
    account_to_map: HashMap<String, String>,
) -> Result<(), Error> {
    let email_conf: EmailConfig = get_smtp_email_config(app_conf);

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
        &email_conf,
        class_info.email.as_str(),
        title.as_str(),
        account_htm.as_str(),
    )
    .await?;

    info!("email title: {} send success", title);

    Ok(())
}

// 从配置文件中读取邮箱服务配置
fn get_smtp_email_config(app_conf: web::Data<AppConfig>) -> EmailConfig {
    EmailConfig {
        smtp_server: app_conf.smtp.0.to_string(),
        smtp_port: app_conf.smtp.1,
        username: app_conf.smtp.2.to_string(),
        password: app_conf.smtp.3.to_string(),
        from_name: app_conf.smtp.4.to_string(),
        from_email: app_conf.smtp.5.to_string(),
    }
}

// 班级内学生账户列表
pub async fn list(
    app_conf: web::Data<AppConfig>,
    class_id: i64,
    user_info: TeacherUserInfo,
) -> Result<Vec<ClassStudentResp>, Error> {
    let db = &app_conf.db;

    check_class_info(db, class_id, user_info.0.user_id, false).await?;

    let rows = ClassStudent::find_by_class_id(db, class_id)
        .await
        .map_err(|err| {
            error!("Select class err: {}", err);
            Error::new(ErrorKind::InvalidInput, "班级账号查询错误")
        })?;

    Ok(rows.into_iter().map(to_info_resp).collect())
}

fn to_info_resp(raw: ClassStudent) -> ClassStudentResp {
    ClassStudentResp {
        id: raw.id,
        class_id: raw.class_id,
        account: raw.account,
        status: raw.status,
        status_desc: StudentStatus::desc(raw.status),
        remark: raw.remark,
        created_at: to_local_datetime(raw.created_at.unwrap_or_default()),
        updated_at: to_local_datetime(raw.updated_at.unwrap_or_default()),
    }
}

// 编辑用户信息
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: ClassStudentEditReq,
    user_info: TeacherUserInfo,
) -> Result<bool, Error> {
    validate_student_edit_req(&req)?;

    let account = req.account.clone().trim().to_string();

    let db = &app_conf.db;

    let class_row = check_class_info(db, req.class_id, user_info.0.user_id, true).await?;

    let student = ClassStudent::find_by_id(db, req.id)
        .await
        .map_err(|err| {
            error!("Select class student {} err: {}", req.id, err);
            Error::new(ErrorKind::InvalidInput, "查询学生账户信息错误")
        })?
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "学生账户不存在"))?;

    // 检查学生账户是否可编辑
    check_student_is_edit(db, &account, &student).await?;

    let mut edit_req: ClassStudent = ClassStudent {
        id: student.id,
        class_id: req.class_id,
        account: account.clone(),
        password: student.password.clone(),
        status: StudentStatus::from_i16(req.status).as_i16(),
        remark: req.remark,
        created_at: None,
        updated_at: None,
    };

    // 生成密码
    let mut password: String = "".to_string();
    if req.reset_pwd {
        password = generate_random_password();
        let hashed = hash_password(&app_conf.student_pepper, &password).map_err(|err| {
            error!(
                "Generate student account {} password err: {}",
                req.account, err
            );
            Error::new(ErrorKind::InvalidInput, "生成学生密码失败, 请重试")
        })?;

        // 更细密码
        edit_req.password = hashed;
    }

    // 更新账户信息
    let rows = ClassStudent::update_by_id(db, &edit_req)
        .await
        .map_err(|err| {
            error!("Update student account {} err: {}", req.id, err);
            Error::new(ErrorKind::InvalidInput, "更新学生账户信息失败")
        })?;

    // 如果重置密码则发送通知邮件
    if req.reset_pwd {
        let mut account_to_map: HashMap<String, String> = HashMap::new();
        account_to_map
            .entry(req.account.clone())
            .or_insert(password);
        send_account_email(app_conf, &class_row, account_to_map).await?;
    }

    Ok(rows > 0)
}

// 检查必填参数
fn validate_student_edit_req(req: &ClassStudentEditReq) -> Result<(), Error> {
    if req.class_id <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "班级为空"));
    }
    if req.account.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "账户列表为空"));
    }

    Ok(())
}

// 验证学生账户是否可以编辑
async fn check_student_is_edit(
    db: &PgPool,
    account: &str,
    student: &ClassStudent,
) -> Result<(), Error> {
    // 验证账户是否存在-登录账户必须是全局的唯一
    let has_rows = ClassStudent::find_by_account(db, account)
        .await
        .map_err(|e| {
            Error::new(ErrorKind::Other, format!("{:?}", e));
            Error::new(ErrorKind::InvalidInput, "班级学生查询错误")
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

    Err(Error::new(
        ErrorKind::InvalidInput,
        format!("账户: {} 已存在, 无法修改", account),
    ))
}
