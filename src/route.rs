use actix_web::web;

use crate::api::question;
use crate::api::{config, file};

// config related
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(config::get_guidance)
        .service(config::get_guidance_by_publisher)
        .service(config::get_catalogs)
        .service(config::get_questions)
        .service(config::get_tags);
}

// file related
pub fn file(cfg: &mut web::ServiceConfig) {
    cfg.service(file::upload)
        .service(file::read)
        .service(file::delete);
}

// question
pub fn question(cfg: &mut web::ServiceConfig) {
    cfg.service(question::upload_question)
        .service(question::get_question_info)
        .service(question::get_question_list);
}
