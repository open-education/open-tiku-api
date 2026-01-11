use crate::api::edit::{
    EditAnalyzeReq, EditAnswerReq, EditKnowledgeReq, EditMentionReq, EditProcessReq,
    EditQuestionTypeReq, EditRateReq, EditRemarkReq, EditSelectLayoutReq, EditSelectReq,
    EditTagsReq, EditTitleReq,
};
use crate::model::question::{Question, QuestionOption};
use crate::AppConfig;
use actix_web::web;
use log::error;
use sqlx::types::Json;
use sqlx::PgPool;
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

// 添加图片
pub async fn add_image(pool: &PgPool, id: i64, image_name: &str) -> Result<bool, Error> {
    if id <= 0 {
        return Ok(true);
    }

    let info = Question::find_by_id(pool, id).await.map_err(|e| {
        error!("Error while removing image: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    let mut images = info.images.unwrap_or_default().0;
    images.push(image_name.to_string());

    edit_images(pool, id, images).await
}

// 删除图片
pub async fn remove_image(pool: &PgPool, id: i64, image_name: &str) -> Result<bool, Error> {
    if id <= 0 {
        return Ok(true);
    }

    let info = Question::find_by_id(pool, id).await.map_err(|e| {
        error!("Error while removing image: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    let mut images = info.images.unwrap_or_default().0;
    images.retain(|x| x != image_name);

    edit_images(pool, id, images).await
}

// 更新题目图片地址
async fn edit_images(pool: &PgPool, id: i64, images: Vec<String>) -> Result<bool, Error> {
    let row = Question::update_images_by_id(pool, id, images)
        .await
        .map_err(|e| {
            error!("Error while updating Image: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新标题补充说明
pub async fn edit_mention(
    app_conf: web::Data<AppConfig>,
    req: EditMentionReq,
) -> Result<bool, Error> {
    let row = Question::update_comment_by_id(&app_conf.get_ref().db, req.id, req.mention)
        .await
        .map_err(|e| {
            error!("Error while updating Comment: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 更新选项样式
pub async fn edit_options_layout(
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

// 合并选项
fn merge_options(
    source: Option<Json<Vec<QuestionOption>>>,
    req: EditSelectReq,
) -> Vec<QuestionOption> {
    // 1. 提取原始数据，直接解构 Json
    let mut options = source.map(|j| j.0).unwrap_or_default();

    // 2. 预校验：新项是否有实际内容（优化：提前计算避免重复判断）
    let is_new_valid = !req.option.content.is_empty()
        || req
            .option
            .images
            .as_ref()
            .is_some_and(|images| !images.is_empty());

    // 3. 查找是否存在相同 label 的项
    // position 返回第一个匹配项的下标
    if let Some(index) = options.iter().position(|opt| opt.label == req.option.label) {
        if is_new_valid {
            // 场景 A：已存在且有效 -> 更新该位置
            options[index] = req.option;
        } else {
            // 场景 B：已存在但新项为空 -> 移除该项
            options.remove(index);
        }
    } else if is_new_valid {
        // 场景 C：不存在且有效 -> 直接追加
        options.push(req.option);
    }

    // 4. 排序：针对 i16 使用不稳定排序性能更佳
    options.sort_unstable_by_key(|q| q.order);

    options
}

// 编辑选项
pub async fn edit_options(
    app_conf: web::Data<AppConfig>,
    req: EditSelectReq,
) -> Result<bool, Error> {
    // todo 暂时不加事务
    let db = &app_conf.get_ref().db;

    let info = Question::find_by_id(db, req.id).await.map_err(|err| {
        error!("edit question get by id err: {:?}", err);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    let row = Question::update_options_by_id(db, req.id, merge_options(info.options, req))
        .await
        .map_err(|e| {
            error!("Error while updating Options: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑参考答案
pub async fn edit_answer(
    app_conf: web::Data<AppConfig>,
    req: EditAnswerReq,
) -> Result<bool, Error> {
    let row = Question::update_answer_by_id(&app_conf.get_ref().db, req.id, req.answer)
        .await
        .map_err(|e| {
            error!("Error while updating Answer: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑知识点
pub async fn edit_knowledge(
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
pub async fn edit_analyze(
    app_conf: web::Data<AppConfig>,
    req: EditAnalyzeReq,
) -> Result<bool, Error> {
    let row = Question::update_analysis_by_id(&app_conf.get_ref().db, req.id, req.analyze)
        .await
        .map_err(|e| {
            error!("Error while updating Analysis: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑解题过程
pub async fn edit_process(
    app_conf: web::Data<AppConfig>,
    req: EditProcessReq,
) -> Result<bool, Error> {
    let row = Question::update_process_by_id(&app_conf.get_ref().db, req.id, req.process)
        .await
        .map_err(|e| {
            error!("Error while updating Process: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}

// 编辑备注
pub async fn edit_remark(
    app_conf: web::Data<AppConfig>,
    req: EditRemarkReq,
) -> Result<bool, Error> {
    let row = Question::update_remark_by_id(&app_conf.get_ref().db, req.id, req.remark)
        .await
        .map_err(|e| {
            error!("Error while updating Remark: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;

    Ok(row > 0)
}
