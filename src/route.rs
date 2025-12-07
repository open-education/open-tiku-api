use actix_web::web;

use crate::api::{config, edit, file, question};

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

// 编辑问题, 考虑到冲突将其拆分到尽可能小的片段
pub fn edit(cfg: &mut web::ServiceConfig) {
    cfg.service(edit::edit_question_type)
        .service(edit::edit_tags)
        .service(edit::edit_rate)
        .service(edit::edit_title)
        .service(edit::edit_mention)
        .service(edit::edit_a)
        .service(edit::edit_b)
        .service(edit::edit_c)
        .service(edit::edit_d)
        .service(edit::edit_e)
        .service(edit::edit_answer)
        .service(edit::edit_knowledge)
        .service(edit::edit_analyze)
        .service(edit::edit_process)
        .service(edit::edit_remark);
}
