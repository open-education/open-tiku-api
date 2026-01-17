use crate::AppConfig;
use crate::api::edit::{
    EditAnalyzeReq, EditAnswerReq, EditImageReq, EditKnowledgeReq, EditMentionReq, EditProcessReq,
    EditQuestionTypeReq, EditRateReq, EditRemarkReq, EditSelectLayoutReq, EditSelectReq,
    EditStatusReq, EditTagsReq, EditTitleReq,
};
use crate::model::question::Question;
use actix_web::web;
use log::error;
use std::io::{Error, ErrorKind};

// 更新题目类型
pub async fn question_type(
    app_conf: web::Data<AppConfig>,
    req: EditQuestionTypeReq,
) -> Result<bool, Error> {
    let db = &app_conf.get_ref().db;
    let row = Question::update_question_type_by_id(db, req.id, req.question_type)
        .await
        .map_err(|e| {
            error!("Error while updating QuestionType: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新题目标签
pub async fn tags(app_conf: web::Data<AppConfig>, req: EditTagsReq) -> Result<bool, Error> {
    let row = Question::update_question_tags_by_id(&app_conf.get_ref().db, req.id, req.tags)
        .await
        .map_err(|e| {
            error!("Error while updating QuestionType: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新题目难易程度
pub async fn rate(app_conf: web::Data<AppConfig>, req: EditRateReq) -> Result<bool, Error> {
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
pub async fn title(app_conf: web::Data<AppConfig>, req: EditTitleReq) -> Result<bool, Error> {
    let row = Question::update_title_by_id(&app_conf.get_ref().db, req.id, req.title)
        .await
        .map_err(|e| {
            error!("Error while updating title: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新题目图片地址
pub async fn images(app_conf: web::Data<AppConfig>, req: EditImageReq) -> Result<bool, Error> {
    let db = &app_conf.get_ref().db;

    let row = Question::update_images_by_id(db, req.id, req.images)
        .await
        .map_err(|e| {
            error!("Error while updating Image: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新标题补充说明
pub async fn mention(app_conf: web::Data<AppConfig>, req: EditMentionReq) -> Result<bool, Error> {
    let row = Question::update_comment_by_id(&app_conf.get_ref().db, req.id, req.mention)
        .await
        .map_err(|e| {
            error!("Error while updating Comment: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新选项样式
pub async fn options_layout(
    app_conf: web::Data<AppConfig>,
    req: EditSelectLayoutReq,
) -> Result<bool, Error> {
    let row = Question::update_options_layout_by_id(&app_conf.get_ref().db, req.id, req.layout)
        .await
        .map_err(|e| {
            error!("Error while updating Options: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑选项
pub async fn options(app_conf: web::Data<AppConfig>, req: EditSelectReq) -> Result<bool, Error> {
    let db = &app_conf.get_ref().db;

    let row = Question::update_options_by_id(db, req.id, req.options)
        .await
        .map_err(|e| {
            error!("Error while updating Options: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑参考答案
pub async fn answer(app_conf: web::Data<AppConfig>, req: EditAnswerReq) -> Result<bool, Error> {
    let row = Question::update_answer_by_id(&app_conf.get_ref().db, req.id, req.answer)
        .await
        .map_err(|e| {
            error!("Error while updating Answer: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑知识点
pub async fn knowledge(
    app_conf: web::Data<AppConfig>,
    req: EditKnowledgeReq,
) -> Result<bool, Error> {
    let row = Question::update_knowledge_by_id(&app_conf.get_ref().db, req.id, req.knowledge)
        .await
        .map_err(|e| {
            error!("Error while updating Knowledge: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑解题分析
pub async fn analyze(app_conf: web::Data<AppConfig>, req: EditAnalyzeReq) -> Result<bool, Error> {
    let db = &app_conf.get_ref().db;

    let row = Question::update_analysis_by_id(db, req.id, req.analyze)
        .await
        .map_err(|e| {
            error!("Error while updating Analysis: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑解题过程
pub async fn process(app_conf: web::Data<AppConfig>, req: EditProcessReq) -> Result<bool, Error> {
    let db = &app_conf.get_ref().db;

    let row = Question::update_process_by_id(db, req.id, req.process)
        .await
        .map_err(|e| {
            error!("Error while updating Process: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑备注
pub async fn remark(app_conf: web::Data<AppConfig>, req: EditRemarkReq) -> Result<bool, Error> {
    let row = Question::update_remark_by_id(&app_conf.get_ref().db, req.id, req.remark)
        .await
        .map_err(|e| {
            error!("Error while updating Remark: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新状态
pub async fn status(app_conf: web::Data<AppConfig>, req: EditStatusReq) -> Result<bool, Error> {
    let approve_id = 1;

    let row = Question::update_status_by_id(
        &app_conf.get_ref().db,
        req.id,
        req.status,
        approve_id,
        req.reject_reason,
    )
    .await
    .map_err(|e| {
        error!("Error while updating Status: {:?}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    Ok(row > 0)
}
