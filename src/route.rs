use actix_web::web;

use crate::api::{chapter_knowledge, edit, file, other_dict, question, question_cate, textbook};

// 图片等资源
pub fn file(cfg: &mut web::ServiceConfig) {
    cfg.service(file::upload)
        .service(file::read)
        .service(file::delete);
}

// 题目
pub fn question(cfg: &mut web::ServiceConfig) {
    cfg.service(question::add)
        .service(question::info)
        .service(question::list);
}

// 编辑问题, 考虑到冲突将其拆分到尽可能小的片段
pub fn edit(cfg: &mut web::ServiceConfig) {
    cfg.service(edit::question_type)
        .service(edit::tags)
        .service(edit::rate)
        .service(edit::title)
        .service(edit::mention)
        .service(edit::options_layout)
        .service(edit::options)
        .service(edit::answer)
        .service(edit::knowledge)
        .service(edit::analyze)
        .service(edit::process)
        .service(edit::remark)
        .service(edit::status);
}

// 教材菜单
pub fn textbook(cfg: &mut web::ServiceConfig) {
    cfg.service(textbook::list_all)
        .service(textbook::list_part)
        .service(textbook::list_children)
        .service(textbook::add)
        .service(textbook::edit)
        .service(textbook::info)
        .service(textbook::delete);
}

// 教材章节和知识点关联
pub fn chapter_knowledge(cfg: &mut web::ServiceConfig) {
    cfg.service(chapter_knowledge::add)
        .service(chapter_knowledge::edit)
        .service(chapter_knowledge::knowledge)
        .service(chapter_knowledge::chapter)
        .service(chapter_knowledge::info);
}

// 教材题型
pub fn question_cate(cfg: &mut web::ServiceConfig) {
    cfg.service(question_cate::list)
        .service(question_cate::add)
        .service(question_cate::remove);
}

// 教材其它字典
pub fn textbook_dict(cfg: &mut web::ServiceConfig) {
    cfg.service(other_dict::add)
        .service(other_dict::remove)
        .service(other_dict::list);
}
