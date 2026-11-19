use actix_web::web;

use crate::api::{
    callback, chapter_knowledge, city, class, class_student, custom, edit, file, homework,
    other_dict, paper, paperv, question, question_cate, stat, task, test, text, textbook, user,
};

// web 服务路由配置

// 图片等资源
pub fn file(cfg: &mut web::ServiceConfig) {
    cfg.service(file::upload_image)
        .service(file::upload_file)
        .service(file::read_file)
        .service(file::read_image)
        .service(file::delete_file);
}

// 题目
pub fn question(cfg: &mut web::ServiceConfig) {
    cfg.service(question::add)
        .service(question::info)
        .service(question::list)
        .service(question::similar)
        .service(question::delete)
        .service(question::count);
}

// 编辑问题, 考虑到冲突将其拆分到尽可能小的片段
pub fn edit(cfg: &mut web::ServiceConfig) {
    cfg.service(edit::question_status)
        .service(edit::paper_status);
}

// 教材菜单
pub fn textbook(cfg: &mut web::ServiceConfig) {
    cfg.service(textbook::list_all)
        .service(textbook::list_level)
        .service(textbook::list_children)
        .service(textbook::add)
        .service(textbook::delete);
}

// 教材章节和知识点关联
pub fn chapter_knowledge(cfg: &mut web::ServiceConfig) {
    cfg.service(chapter_knowledge::add)
        .service(chapter_knowledge::list)
        .service(chapter_knowledge::remove);
}

// 教材题型
pub fn question_cate(cfg: &mut web::ServiceConfig) {
    cfg.service(question_cate::list)
        .service(question_cate::list_all)
        .service(question_cate::add)
        .service(question_cate::remove);
}

// 教材其它字典
pub fn textbook_dict(cfg: &mut web::ServiceConfig) {
    cfg.service(other_dict::add)
        .service(other_dict::remove)
        .service(other_dict::list)
        .service(other_dict::list_all);
}

pub fn task(cfg: &mut web::ServiceConfig) {
    cfg.service(task::add).service(task::list);
}

pub fn paperv(cfg: &mut web::ServiceConfig) {
    cfg.service(paperv::top_add)
        .service(paperv::top_info)
        .service(paperv::list)
        .service(paperv::latest)
        .service(paperv::preview)
        .service(paperv::gen_add)
        .service(paperv::gen_info)
        .service(paperv::delete);
}

pub fn paper(cfg: &mut web::ServiceConfig) {
    cfg.service(paper::upload_add)
        .service(paper::count)
        .service(paper::list);
}

pub fn text(cfg: &mut web::ServiceConfig) {
    cfg.service(text::question_snippet);
}

pub fn callback(cfg: &mut web::ServiceConfig) {
    cfg.service(callback::login_url)
        .service(callback::github)
        .service(callback::qq);
}

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(user::exchange)
        .service(user::login)
        .service(user::info)
        .service(user::logout)
        .service(user::account_list)
        .service(user::session_list)
        .service(user::edit);
}

pub fn class(cfg: &mut web::ServiceConfig) {
    cfg.service(class::add).service(class::list);
}

pub fn class_student(cfg: &mut web::ServiceConfig) {
    cfg.service(class_student::add)
        .service(class_student::list)
        .service(class_student::edit);
}

pub fn homework(cfg: &mut web::ServiceConfig) {
    cfg.service(homework::add)
        .service(homework::batch_no)
        .service(homework::list);
}

pub fn test(cfg: &mut web::ServiceConfig) {
    cfg.service(test::list)
        .service(test::attempt_latest)
        .service(test::answer_add)
        .service(test::attempts);
}

pub fn stat(cfg: &mut web::ServiceConfig) {
    cfg.service(stat::board);
}

pub fn custom(cfg: &mut web::ServiceConfig) {
    cfg.service(custom::add).service(custom::list);
}

pub fn city(cfg: &mut web::ServiceConfig) {
    cfg.service(city::list);
}
