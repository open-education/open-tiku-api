///
/// 配置相关的内容, 目前存在文件中, 并且未做优化, 后续有资源了会将其存入缓存等服务中
/// 当前未直接存入内存, 考虑到尽可能让服务不占用过多的内存从而影响其它服务
///
use crate::AppConfig;
use crate::config::{catalog, guidance, knowledge, question, subject, tag};
use crate::util::response::ApiResponse;
use actix_web::{get, web};

#[get("/get-guidance")]
pub async fn get_guidance(app_conf: web::Data<AppConfig>) -> ApiResponse<Vec<subject::Subject>> {
    ApiResponse::response(guidance::get_guidance(
        app_conf.meta_path.to_str().unwrap_or(""),
    ))
}

#[get("/get-catalogs/{key}")]
pub async fn get_catalogs(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> ApiResponse<Vec<catalog::Catalog>> {
    ApiResponse::response(catalog::get_catalogs(
        app_conf.meta_path.to_str().unwrap_or(""),
        &path.into_inner().0,
    ))
}

#[get("/get-question-types/{key}")]
pub async fn get_questions(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> ApiResponse<Vec<question::QuestionType>> {
    ApiResponse::response(question::get_question_types(
        app_conf.meta_path.to_str().unwrap_or(""),
        &path.into_inner().0,
    ))
}

#[get("/get-tags/{key}")]
pub async fn get_tags(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> ApiResponse<Vec<tag::Tag>> {
    ApiResponse::response(tag::get_tags(
        app_conf.meta_path.to_str().unwrap_or(""),
        &path.into_inner().0,
    ))
}

#[get("/get-knowledge-info/{key}")]
pub async fn get_knowledge_info(
    app_conf: web::Data<AppConfig>,
    path: web::Path<(String,)>,
) -> ApiResponse<Vec<knowledge::KnowledgeInfo>> {
    ApiResponse::response(knowledge::get_knowledge_info(
        app_conf.meta_path.to_str().unwrap_or(""),
        &path.into_inner().0,
    ))
}
