use crate::api::edit::{
    EditAReq, EditAnalyzeReq, EditAnswerReq, EditBReq, EditCReq, EditDReq, EditEReq,
    EditKnowledgeReq, EditMentionReq, EditProcessReq, EditQuestionTypeReq, EditRateReq,
    EditRemarkReq, EditSelectReq, EditTagsReq, EditTitleReq,
};
use crate::model::question::Question;
use crate::AppConfig;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 更新题目类型
pub async fn edit_question_type(
    app_conf: web::Data<AppConfig>,
    req: EditQuestionTypeReq,
) -> Result<bool, Error> {
    let row =
        Question::update_question_type_by_id(&app_conf.get_ref().db, req.id, req.question_type)
            .await
            .map_err(|e| {
                error!("Error while updating QuestionType: {:?}", e);
                Error::new(ErrorKind::Other, "更新失败")
            })?;

    Ok(row > 0)
}

// 更新题目标签
pub async fn edit_tags(app_conf: web::Data<AppConfig>, req: EditTagsReq) -> Result<bool, Error> {
    let row = Question::update_question_tags_by_id(&app_conf.get_ref().db, req.id, req.tags)
        .await
        .map_err(|e| {
            error!("Error while updating QuestionType: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新题目难易程度
pub async fn edit_rate(app_conf: web::Data<AppConfig>, req: EditRateReq) -> Result<bool, Error> {
    let row = Question::update_difficulty_level_by_id(
        &app_conf.get_ref().db,
        req.id,
        req.difficulty_level,
    )
    .await
    .map_err(|e| {
        error!("Error while updating DifficultyLevel: {:?}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    Ok(row > 0)
}

// 更新标题
pub async fn edit_title(app_conf: web::Data<AppConfig>, req: EditTitleReq) -> Result<bool, Error> {
    let row = Question::update_title_by_id(&app_conf.get_ref().db, req.id, req.title)
        .await
        .map_err(|e| {
            error!("Error while updating title: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

pub fn edit_select(app_conf: web::Data<AppConfig>, req: EditSelectReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_mention(app_conf: web::Data<AppConfig>, req: EditMentionReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_a(app_conf: web::Data<AppConfig>, req: EditAReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_b(app_conf: web::Data<AppConfig>, req: EditBReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_c(app_conf: web::Data<AppConfig>, req: EditCReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_d(app_conf: web::Data<AppConfig>, req: EditDReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_e(app_conf: web::Data<AppConfig>, req: EditEReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_answer(app_conf: web::Data<AppConfig>, req: EditAnswerReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_knowledge(
    app_conf: web::Data<AppConfig>,
    req: EditKnowledgeReq,
) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_analyze(app_conf: web::Data<AppConfig>, req: EditAnalyzeReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_process(app_conf: web::Data<AppConfig>, req: EditProcessReq) -> Result<bool, Error> {
    Ok(true)
}

pub fn edit_remark(app_conf: web::Data<AppConfig>, req: EditRemarkReq) -> Result<bool, Error> {
    Ok(true)
}
