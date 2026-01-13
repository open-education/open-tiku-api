use crate::model::question::{Content, QuestionOption};
use crate::service::question;
use crate::util::response::ApiResponse;
use crate::AppConfig;
use actix_web::{get, post, web};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

// 添加题目请求
#[derive(Deserialize)]
pub struct CreateQuestionReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i32, // 题型主键
    #[serde(rename(deserialize = "sourceId"))]
    pub source_id: Option<i64>, // 变式题父主键
    #[serde(rename(deserialize = "questionTypeId"))]
    pub question_type_id: i32, // 题型类型主键
    #[serde(rename(deserialize = "questionTagIds"))]
    pub question_tag_ids: Option<Vec<i32>>, // 题型标签主键
    pub author_id: Option<i64>, // 作者, 内部逻辑生成

    pub title: String,                 // 标题
    pub content_plain: Option<String>, // 去除公式等特殊字符的标题, 为了搜索用, 内部逻辑生成
    pub comment: Option<String>,       // 标题补充说明

    // 使用 rust_decimal 处理 0.5 精度问题
    #[serde(rename(deserialize = "difficultyLevel"))]
    pub difficulty_level: Decimal, // 题目难易程度

    pub images: Option<Json<Vec<String>>>, // 题目图片列表

    pub options: Option<Json<Vec<QuestionOption>>>, // 选项内容
    #[serde(rename(deserialize = "optionsLayout"))]
    pub options_layout: Option<i16>, // 使用 i16 对应数据库 SMALLINT

    // 答案与解析
    pub answer: Option<String>,          // 参考答案
    pub knowledge: Option<String>,       // 知识点文本描述
    pub analysis: Option<Json<Content>>, // 解题分析
    pub process: Option<Json<Content>>,  // 解题过程
    pub remark: Option<String>,          // 备注
}

// 添加题目
#[post("/add")]
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: web::Json<CreateQuestionReq>,
) -> ApiResponse<i64> {
    ApiResponse::response(question::add(app_conf, req.into_inner()).await)
}

// 题库基本信息返回
#[derive(Serialize)]
pub struct QuestionBaseResp {
    pub id: i64,
    #[serde(rename(serialize = "questionCateId"))]
    pub question_cate_id: i32, // 题型主键
    #[serde(rename(serialize = "questionTypeId"))]
    pub question_type_id: i32, // 题型类型主键
    #[serde(rename(serialize = "questionTagIds"))]
    pub question_tag_ids: Option<Json<Vec<i32>>>, // 题型标签主键
    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64, // 作者, 内部逻辑生成

    pub title: String, // 标题
    #[serde(rename(serialize = "contentPlain"))]
    pub content_plain: Option<String>, // 去除公式等特殊字符的标题, 为了搜索用, 内部逻辑生成
    pub comment: Option<String>, // 标题补充说明

    // 使用 rust_decimal 处理 0.5 精度问题
    #[serde(rename(serialize = "difficultyLevel"))]
    pub difficulty_level: Decimal, // 题目难易程度

    pub images: Option<Json<Vec<String>>>, // 题目图片列表

    pub options: Option<Json<Vec<QuestionOption>>>, // 选项内容
    #[serde(rename(serialize = "optionsLayout"))]
    pub options_layout: Option<i16>, // 使用 i16 对应数据库 SMALLINT

    // 题目详情信息列表不返回, 需要再返回

    // 审核相关
    pub status: i16, // 审核状态
    #[serde(rename(serialize = "approveId"))]
    pub approve_id: i64, // 审核人
    #[serde(rename(serialize = "rejectReason"))]
    pub reject_reason: Option<String>, // 拒绝原因
    #[serde(rename(serialize = "approveAt"))]
    pub approve_at: Option<chrono::DateTime<chrono::Utc>>, // 审核时间

    // 创建更新时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// 其它额外信息, 后续非列表字段再这里补充
#[derive(Serialize)]
pub struct QuestionExtraInfo {
    pub answer: Option<String>,
    pub knowledge: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub process: Option<Json<Content>>,
    pub remark: Option<String>,
}

#[derive(Serialize)]
pub struct QuestionInfoResp {
    #[serde(rename(serialize = "baseInfo"))]
    pub base_info: QuestionBaseResp,
    #[serde(rename(serialize = "extraInfo"))]
    pub extra_info: QuestionExtraInfo,
}

#[get("/info/{id}")]
pub async fn info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(i64,)>,
) -> ApiResponse<QuestionInfoResp> {
    ApiResponse::response(question::info(app_conf, path.into_inner().0).await)
}

#[derive(Deserialize)]
pub struct QuestionListReq {
    #[serde(rename(deserialize = "questionCateId"))]
    pub question_cate_id: i32,
    #[serde(rename(deserialize = "questionTypeId"))]
    pub question_type_id: Option<i32>,
    pub status: Option<i16>,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i32,
}

#[derive(Serialize)]
pub struct QuestionListResp {
    pub list: Vec<QuestionBaseResp>,
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: i32,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: i32,
    pub total: i64,
}

// 题目列表
#[post("/list")]
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<QuestionListReq>,
) -> ApiResponse<QuestionListResp> {
    ApiResponse::response(question::list(app_conf, req.into_inner()).await)
}
