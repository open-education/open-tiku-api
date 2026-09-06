use crate::model::question::{Content, QuestionOption, Step};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::types::Json;

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
    #[serde(rename(serialize = "questionDimensionIds"))]
    pub question_dimension_ids: Option<Json<Vec<i32>>>, // 核心素养
    #[serde(rename(serialize = "relationType"))]
    pub relation_type: i16, // 题目类型
    #[serde(rename(serialize = "authorId"))]
    pub author_id: i64, // 作者, 内部逻辑生成
    #[serde(rename(serialize = "authorName"))]
    pub author_name: String, // 作者昵称
    pub source: String,
    #[serde(rename(serialize = "originalName"))]
    pub original_name: String,

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
    #[serde(rename(serialize = "approveName"))]
    pub approve_name: String,
    #[serde(rename(serialize = "rejectReason"))]
    pub reject_reason: Option<String>, // 拒绝原因
    #[serde(rename(serialize = "approveAt"))]
    pub approve_at: String, // 审核时间

    pub steps: Option<Json<Vec<Step>>>, // 解题步骤需要返回

    // 创建更新时间
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: String,
    #[serde(rename(serialize = "updatedAt"))]
    pub updated_at: String,
}

// 其它额外信息, 后续非列表字段再这里补充
#[derive(Serialize)]
pub struct QuestionExtraInfoResp {
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
    pub extra_info: QuestionExtraInfoResp,
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
