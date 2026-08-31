use crate::api::req::class::ClassInfoReq;
use crate::api::resp::class::{ClassListReq, ClassListResp};
use crate::app::conf::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::model::class::Class;
use crate::util::error::AppError;
use tracing::error;

// 添加班级
pub async fn add(
    app_state: &AppState,
    req: ClassInfoReq,
    user_info: TeacherUserInfo,
) -> Result<i64, AppError> {
    validate_class_req(&req)?;

    let db = &app_state.db;

    if req.id.is_some() {
        let has = Class::find_by_id(db, req.id.clone().unwrap_or_default())
            .await
            .map_err(|err| {
                error!("Select class err: {}", err);
                AppError::db_error("查询班级信息错误")
            })?
            .ok_or_else(|| AppError::not_found("班级不存在"))?;
        if has.author_id != user_info.0.user_id {
            return Err(AppError::permission_denied("只允许编辑自己的班级"));
        }
    }

    let class = build_class_req(req, user_info.0.user_id);
    let row_id = Class::save(db, class).await.map_err(|err| {
        error!("Save class err: {}", err);
        AppError::db_error("班级创建失败")
    })?;

    Ok(row_id)
}

// 验证请求参数
fn validate_class_req(req: &ClassInfoReq) -> Result<(), AppError> {
    if req.year.is_empty() {
        return Err(AppError::param_error("年份不能为空"));
    }
    if req.label.is_empty() {
        return Err(AppError::param_error("班级名称不能为空"));
    }

    Ok(())
}

// 转化请求对象模型对象
fn build_class_req(req: ClassInfoReq, user_id: i64) -> Class {
    Class {
        id: req.id,
        year: req.year,
        grade: req.grade.unwrap_or_default(),
        semester: req.semester.unwrap_or_default(),
        label: req.label,
        email: req.email,
        sort_order: req.sort_order,
        author_id: user_id,
        remark: req.remark,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// 班级列表
pub async fn list(
    app_state: &AppState,
    req: ClassListReq,
    user_info: TeacherUserInfo,
) -> Result<ClassListResp, AppError> {
    let db = &app_state.db;

    let count = Class::count(db, user_info.0.user_id, &req)
        .await
        .map_err(|err| {
            error!("Class count err: {}", err);
            AppError::db_error("班级计数信息查询失败")
        })?;

    let offset = (req.page_no - 1) * req.page_size;
    if offset >= count as i32 {
        return Ok(ClassListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total: count,
        });
    }

    let rows = Class::list(db, user_info.0.user_id, &req, offset)
        .await
        .map_err(|err| {
            error!("Select class err: {}", err);
            AppError::db_error("班级信息查询失败")
        })?;

    Ok(ClassListResp {
        list: rows.into_iter().map(Into::into).collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total: count,
    })
}
