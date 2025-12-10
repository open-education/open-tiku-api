use crate::AppConfig;
use crate::service::question;
use crate::util::response::ApiResponse;
use actix_web::{post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct QuestionUploadReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    #[serde(rename(deserialize = "sourceId"))]
    pub source_id: Option<String>,
    #[serde(rename(deserialize = "questionType"))]
    pub question_type: String,
    pub tags: Option<Vec<String>>,
    #[serde(rename(deserialize = "rateVal"))]
    pub rate_val: Option<String>,
    #[serde(rename(deserialize = "titleVal"))]
    pub title_val: String,
    #[serde(rename(deserialize = "mentionVal"))]
    pub mention_val: Option<String>,
    #[serde(rename(deserialize = "imageNames"))]
    pub image_names: Option<Vec<String>>,
    #[serde(rename(deserialize = "showImageVal"))]
    pub show_image_val: Option<String>,
    #[serde(rename(deserialize = "aVal"))]
    pub a_val: Option<String>,
    #[serde(rename(deserialize = "bVal"))]
    pub b_val: Option<String>,
    #[serde(rename(deserialize = "cVal"))]
    pub c_val: Option<String>,
    #[serde(rename(deserialize = "dVal"))]
    pub d_val: Option<String>,
    #[serde(rename(deserialize = "eVal"))]
    pub e_val: Option<String>,
    #[serde(rename(deserialize = "showSelectVal"))]
    pub show_select_val: Option<String>,
    #[serde(rename(deserialize = "answerVal"))]
    pub answer_val: Option<String>,
    #[serde(rename(deserialize = "knowledgeVal"))]
    pub knowledge_val: Option<String>,
    #[serde(rename(deserialize = "analyzeVal"))]
    pub analyze_val: Option<String>,
    #[serde(rename(deserialize = "processVal"))]
    pub process_val: Option<String>,
    #[serde(rename(deserialize = "remarkVal"))]
    pub remark_val: Option<String>,
}

#[derive(Serialize)]
pub struct QuestionUploadResp {
    pub id: String,
}

#[post("/upload")]
pub async fn upload_question(
    app_conf: web::Data<AppConfig>,
    req: web::Json<QuestionUploadReq>,
) -> ApiResponse<QuestionUploadResp> {
    ApiResponse::response(
        question::upload_question(app_conf.meta_path.to_str().unwrap(), req.into_inner()).await,
    )
}

#[derive(Deserialize)]
pub struct QuestionInfoReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    pub id: String,
    pub ext: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct QuestionInfoResp {
    pub id: String,
    #[serde(rename(serialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(serialize = "catalogKey"))]
    pub catalog_key: String,
    #[serde(rename(serialize = "questionType"))]
    pub question_type: String,
    pub tags: Option<Vec<String>>,
    #[serde(rename(serialize = "rateVal"))]
    pub rate_val: Option<String>,
    #[serde(rename(serialize = "titleVal"))]
    pub title_val: String,
    #[serde(rename(serialize = "mentionVal"))]
    pub mention_val: Option<String>,
    #[serde(rename(serialize = "imageNames"))]
    pub image_names: Option<Vec<String>>,
    #[serde(rename(serialize = "showImageVal"))]
    pub show_image_val: Option<String>,
    #[serde(rename(serialize = "aVal"))]
    pub a_val: Option<String>,
    #[serde(rename(serialize = "bVal"))]
    pub b_val: Option<String>,
    #[serde(rename(serialize = "cVal"))]
    pub c_val: Option<String>,
    #[serde(rename(serialize = "dVal"))]
    pub d_val: Option<String>,
    #[serde(rename(serialize = "eVal"))]
    pub e_val: Option<String>,
    #[serde(rename(serialize = "showSelectVal"))]
    pub show_select_val: Option<String>,
    #[serde(rename(serialize = "answerVal"))]
    pub answer_val: Option<String>,
    #[serde(rename(serialize = "knowledgeVal"))]
    pub knowledge_val: Option<String>,
    #[serde(rename(serialize = "analyzeVal"))]
    pub analyze_val: Option<String>,
    #[serde(rename(serialize = "processVal"))]
    pub process_val: Option<String>,
    #[serde(rename(serialize = "remarkVal"))]
    pub remark_val: Option<String>,
}

#[post("/info")]
pub async fn get_question_info(
    app_conf: web::Data<AppConfig>,
    req: web::Json<QuestionInfoReq>,
) -> ApiResponse<QuestionInfoResp> {
    ApiResponse::response(question::get_question_info(
        app_conf.meta_path.to_str().unwrap(),
        req.into_inner(),
    ))
}

#[derive(Deserialize)]
pub struct QuestionListReq {
    #[serde(rename(deserialize = "textbookKey"))]
    pub textbook_key: String,
    #[serde(rename(deserialize = "catalogKey"))]
    pub catalog_key: String,
    #[serde(rename(deserialize = "pageNo"))]
    pub page_no: usize,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: usize,
}

#[derive(Serialize)]
pub struct QuestionListResp {
    #[serde(rename(serialize = "pageNo"))]
    pub page_no: usize,
    #[serde(rename(serialize = "pageSize"))]
    pub page_size: usize,
    pub total: usize,
    pub data: Vec<QuestionInfoResp>,
}

/// 当前仅支持小节目录的题目列表查询, 后续如果有要求查看父级目录下的题目时看索引文件如何设计更适合后在提供
#[post("/list")]
pub async fn get_question_list(
    app_conf: web::Data<AppConfig>,
    req: web::Json<QuestionListReq>,
) -> ApiResponse<QuestionListResp> {
    ApiResponse::response(question::get_question_list(
        app_conf.meta_path.to_str().unwrap(),
        req.into_inner(),
    ))
}
