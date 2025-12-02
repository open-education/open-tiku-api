///
/// 配置相关的内容, 目前存在文件中, 并且未做优化, 后续有资源了会将其存入缓存等服务中
/// 当前未直接存入内存, 考虑到尽可能让服务不占用过多的内存从而影响其它服务
///
use crate::config::catalog::Catalog;
use crate::config::guidance;
use crate::config::publisher::{Publisher, Subject};
use crate::config::question::QuestionType;
use crate::config::tag::Tag;
use crate::config::{catalog, question, tag};
use crate::util::response::ApiResponse;
use actix_web::{get, web};

#[get("/get-guidance")]
pub async fn get_guidance() -> ApiResponse<Vec<Publisher>> {
    ApiResponse::response(guidance::get_guidance())
}

#[get("/get-guidance-by-publisher/{key}")]
pub async fn get_guidance_by_publisher(path: web::Path<(String,)>) -> ApiResponse<Vec<Subject>> {
    ApiResponse::response(guidance::get_guidance_by_publisher(&path.into_inner().0))
}

#[get("/get-catalogs/{key}")]
pub async fn get_catalogs(path: web::Path<(String,)>) -> ApiResponse<Vec<Catalog>> {
    ApiResponse::response(catalog::get_catalogs(&path.into_inner().0))
}

#[get("/get-question-types/{key}")]
pub async fn get_questions(path: web::Path<(String,)>) -> ApiResponse<Vec<QuestionType>> {
    ApiResponse::response(question::get_question_types(&path.into_inner().0))
}

#[get("/get-tags/{key}")]
pub async fn get_tags(path: web::Path<(String,)>) -> ApiResponse<Vec<Tag>> {
    ApiResponse::response(tag::get_tags(&path.into_inner().0))
}
