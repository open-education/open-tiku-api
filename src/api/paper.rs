use crate::AppConfig;
use crate::model::question::{Content, QuestionOption};
use crate::service::paper;
use crate::util::response::ApiResponse;
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use crate::middleware::user::UserInfo;

/// 试卷相关操作

#[derive(Deserialize)]
pub struct PaperReq {
    pub id: Option<i64>,
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(deserialize = "relatedName"))]
    pub related_name: String,
    pub tag: String,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub title: String,
    pub score: i32,
    pub status: i16,
    pub source: String,
    pub remark: Option<String>,
    pub groups: Vec<PaperGroupReq>,
}

#[derive(Deserialize)]
pub struct PaperGroupReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(deserialize = "subTitle"))]
    pub sub_title: Option<String>,
    pub questions: Vec<PaperQuestionReq>,
}

#[derive(Deserialize)]
pub struct PaperQuestionReq {
    #[serde(rename(deserialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(deserialize = "orderNum"))]
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    #[serde(rename(deserialize = "optionsLayout"))]
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

// 添加试卷
#[post("/add")]
pub async fn add(app_conf: web::Data<AppConfig>, req: web::Json<PaperReq>, user_info: UserInfo) -> ApiResponse<i64> {
    ApiResponse::response(paper::add(app_conf, req.into_inner(), user_info).await)
}

// 查看详情
#[derive(Serialize)]
pub struct PaperResp {
    pub id: Option<i64>,
    #[serde(rename(serialize = "relatedId"))]
    pub related_id: i32,
    #[serde(rename(serialize = "relatedName"))]
    pub related_name: String,
    pub tag: String,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub title: String,
    pub score: i32,
    pub source: String,

    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64,
    #[serde(rename(serialize = "authorName"))]
    pub author_name: String,

    // 审核相关
    pub status: i16, // 审核状态
    #[serde(rename(deserialize = "statusDesc"))]
    pub status_desc: String,
    #[serde(rename(serialize = "approveId"))]
    pub approve_id: i64, // 审核人
    #[serde(rename(serialize = "rejectReason"))]
    pub reject_reason: Option<String>, // 拒绝原因
    #[serde(rename(serialize = "approveAt"))]
    pub approve_at: Option<String>, // 审核时间

    pub remark: Option<String>,
    pub count: i32,
    pub groups: Vec<PaperGroupResp>,

    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct PaperGroupResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "typeName"))]
    pub type_name: String,
    #[serde(rename(serialize = "subTitle"))]
    pub sub_title: Option<String>,
    pub questions: Vec<PaperQuestionResp>,
}

#[derive(Serialize)]
pub struct PaperQuestionResp {
    pub id: i64,
    #[serde(rename(serialize = "paperId"))]
    pub paper_id: i64,
    #[serde(rename(serialize = "groupId"))]
    pub group_id: i64,
    #[serde(rename(serialize = "genId"))]
    pub gen_id: String,
    #[serde(rename(serialize = "orderNum"))]
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    #[serde(rename(serialize = "optionsLayout"))]
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

#[get("/info/{id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i64,)>,
) -> ApiResponse<PaperResp> {
    ApiResponse::response(paper::info(app_conf, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct PaperListReq {
    #[serde(rename(deserialize = "relatedId"))]
    pub related_id: i32,
    pub tag: Option<String>,
    pub year: Option<String>,
    pub grade: Option<String>,
    pub semester: Option<String>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct PaperListResp {
    pub list: Vec<PaperResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

#[post("/list")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<PaperListReq>,
) -> ApiResponse<PaperListResp> {
    ApiResponse::response(paper::list(app_conf, req.into_inner()).await)
}

#[get("/latest/{count}")]
pub async fn latest(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i64,)>,
) -> ApiResponse<Vec<PaperResp>> {
    ApiResponse::response(paper::latest(app_conf, path.into_inner().0).await)
}
