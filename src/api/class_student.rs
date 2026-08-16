use crate::app::config::AppState;
use crate::middleware::user::TeacherUserInfo;
use crate::service::class_student;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ClassStudentReq {
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    // 是否增量导入
    pub incremental: bool,
    // 账户名称是英文逗号分割的字符串
    pub accounts: String,
}

// 添加学生账户
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppState>,
    req: web::Json<ClassStudentReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<u64> {
    ApiResponse::response(class_student::add(app_conf, req.into_inner(), user_info).await)
}

#[derive(Serialize)]
pub struct ClassStudentResp {
    pub id: i64,
    #[serde(rename(serialize = "classId"))]
    pub class_id: i64,
    pub user_id: i64,
    pub account: String,
    pub status: i16, // 1 正常 2 暂停 3 停用
    #[serde(rename(serialize = "statusDesc"))]
    pub status_desc: String,
    pub remark: String,
    #[serde(rename(serialize = "lastLoginTime"))]
    pub last_login_time: String,
    #[serde(rename(serialize = "loginCount"))]
    pub login_count: i64,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

// 获取班级的学生账户-不分页直接展示全部
#[get("/{class_id}/list")]
pub async fn list(
    app_conf: web::Data<AppState>,
    path: web::Path<(i64,)>,
    user_info: TeacherUserInfo,
) -> ApiResponse<Vec<ClassStudentResp>> {
    ApiResponse::response(class_student::list(app_conf, path.into_inner().0, user_info).await)
}

// 修改学生账户信息
#[derive(Deserialize)]
pub struct ClassStudentEditReq {
    pub id: i64,
    #[serde(rename(deserialize = "classId"))]
    pub class_id: i64,
    pub account: String,
    pub status: i16,
    #[serde(rename(deserialize = "resetPwd"))]
    pub reset_pwd: bool,
    pub remark: String,
}

// 编辑学生账户信息
#[post("/edit")]
pub async fn edit(
    app_conf: web::Data<AppState>,
    req: web::Json<ClassStudentEditReq>,
    user_info: TeacherUserInfo,
) -> ApiResponse<bool> {
    ApiResponse::response(class_student::edit(app_conf, req.into_inner(), user_info).await)
}
