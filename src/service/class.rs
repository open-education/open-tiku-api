use crate::AppConfig;
use crate::api::class::{ClassInfoReq, ClassInfoResp, ClassListReq, ClassListResp};
use crate::middleware::user::TeacherUserInfo;
use crate::model::class::Class;
use crate::util::local::to_local_datetime;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 添加班级
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: ClassInfoReq,
    user_info: TeacherUserInfo,
) -> Result<i64, Error> {
    validate_class_req(&req)?;

    let db = &app_conf.db;

    if req.id.is_some() {
        let has = Class::find_by_id(db, req.id.clone().unwrap_or_default())
            .await
            .map_err(|err| {
                error!("Select class err: {}", err);
                Error::new(ErrorKind::Other, "查询班级信息错误")
            })?
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "班级不存在"))?;
        if has.author_id != user_info.0.user_id {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "只允许编辑自己的班级",
            ));
        }
    }

    let class = build_class_req(req, user_info.0.user_id);
    let row_id = Class::save(db, class).await.map_err(|err| {
        error!("Save class err: {}", err);
        Error::new(ErrorKind::Other, "班级创建失败")
    })?;

    Ok(row_id)
}

// 验证请求参数
fn validate_class_req(req: &ClassInfoReq) -> Result<(), Error> {
    if req.year.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "年份不能为空"));
    }
    if req.label.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "班级名称不能为空"));
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
        sort_order: req.sort_order,
        author_id: user_id,
        remark: req.remark,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// 班级列表
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: ClassListReq,
    user_info: TeacherUserInfo,
) -> Result<ClassListResp, Error> {
    let db = &app_conf.db;

    let count = Class::count(db, user_info.0.user_id, &req)
        .await
        .map_err(|err| {
            error!("Class count err: {}", err);
            Error::new(ErrorKind::Other, "班级计数信息查询失败")
        })?;

    let rows = Class::list(db, user_info.0.user_id, &req)
        .await
        .map_err(|err| {
            error!("Select class err: {}", err);
            Error::new(ErrorKind::Other, "班级信息查询失败")
        })?;

    Ok(ClassListResp {
        list: rows.into_iter().map(to_info_resp).collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total: count,
    })
}

fn to_info_resp(row: Class) -> ClassInfoResp {
    ClassInfoResp {
        id: row.id.unwrap_or_default(),
        year: row.year,
        grade: row.grade,
        semester: row.semester,
        label: row.label,
        sort_order: row.sort_order,
        remark: row.remark,
        created_at: to_local_datetime(row.created_at),
        updated_at: to_local_datetime(row.updated_at),
    }
}
