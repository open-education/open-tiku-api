use crate::model::question::{Content, QuestionOption, Step};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestionReq {
    pub id: Option<i64>,
    pub question_cate_id: i32,  // 题型主键
    pub source_id: Option<i64>, // 变式题父主键
    // 题目归属类型
    pub relation_type: i16,
    pub question_type_id: i32,                    // 题型类型主键
    pub question_tag_ids: Option<Vec<i32>>,       // 题型标签主键
    pub question_dimension_ids: Option<Vec<i32>>, // 核心素养
    pub author_id: Option<i64>,                   // 作者, 内部逻辑生成
    pub source: String,                           // 来源
    pub original_name: String,                    // 原创者昵称
    pub status: i16,

    pub title: String,                 // 标题
    pub content_plain: Option<String>, // 去除公式等特殊字符的标题, 为了搜索用, 内部逻辑生成
    pub comment: Option<String>,       // 标题补充说明

    // 使用 rust_decimal 处理 0.5 精度问题
    pub difficulty_level: Decimal, // 题目难易程度

    pub images: Option<Json<Vec<String>>>, // 题目图片列表

    pub options: Option<Json<Vec<QuestionOption>>>, // 选项内容
    pub options_layout: Option<i16>,                // 使用 i16 对应数据库 SMALLINT

    // 答案与解析
    pub answer: Option<String>,          // 参考答案
    pub knowledge: Option<String>,       // 知识点文本描述
    pub analysis: Option<Json<Content>>, // 解题分析
    pub process: Option<Json<Content>>,  // 解题过程
    pub steps: Option<Json<Vec<Step>>>,  // 解题步骤
    pub remark: Option<String>,          // 易错备注
    pub remark_ext: Option<String>,      // 其它备注
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionListReq {
    // 页面来源
    pub source: String,
    pub question_cate_ids: Vec<i32>,
    pub question_type_id: Option<i32>,
    pub dimension_ids: Option<Vec<i32>>, // 核心素养
    pub status: Option<i16>,
    pub ids: Option<Vec<i64>>,
    pub title_val: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
    pub page_no: i32,
    pub page_size: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionSimilarListReq {
    pub question_id: i64,
    pub question_cate_id: i32,
    pub question_type_id: Option<i32>,
    pub question_dimension_ids: Option<Vec<i32>>, // 核心素养
    pub status: Option<i16>,
    pub tag_ids: Option<Vec<i32>>,
    pub page_no: i32,
    pub page_size: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalReq {
    pub id: i64,
    pub relation_type: i16,
}

#[derive(Deserialize)]
pub struct DeleteReq {
    pub id: i64,
}
