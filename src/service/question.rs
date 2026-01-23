use crate::api::question::{
    CreateQuestionReq, QuestionBaseResp, QuestionExtraInfo, QuestionInfoResp, QuestionListReq,
    QuestionListResp,
};
use crate::model::question::{Question, QuestionStatus};
use crate::model::question_similar::QuestionSimilar;
use crate::AppConfig;
use actix_web::web;
use log::error;
use regex::Regex;
use std::io::{Error, ErrorKind};

/// 将包含 LaTeX 的富文本标题转换为纯文本
fn to_plain_text(title: &str) -> String {
    // 1. 移除 LaTeX 符号 (例如 $...$ 或 $$...$$)
    let re_latex = Regex::new(r"\$[^$]+\$").unwrap();
    let no_latex = re_latex.replace_all(title, "");

    // 2. 移除 LaTeX 指令 (例如 \frac, \sqrt)
    let re_commands = Regex::new(r"\\[a-zA-Z]+").unwrap();
    let no_commands = re_commands.replace_all(&no_latex, "");

    // 3. 移除多余空白并返回
    no_commands.trim().to_string()
}

// 添加题目
pub async fn add(app_conf: web::Data<AppConfig>, mut req: CreateQuestionReq) -> Result<i64, Error> {
    // 关于重复添加的问题应该要使用 redis 全局锁, 暂时没有 缓存服务
    let db = &app_conf.get_ref().db;

    let source_id = req.source_id;

    // todo 从登录信息中解析出作者
    req.author_id = Some(1);

    req.content_plain = Some(to_plain_text(req.title.as_str()));

    let row = Question::insert(db, req).await.map_err(|e| {
        error!("question add err: {:?}", e);
        Error::new(ErrorKind::Other, "题目添加失败")
    })?;

    // 如果存在变式题则关联
    if source_id.is_some() {
        let _ = QuestionSimilar::insert(db, source_id.unwrap(), row.id)
            .await
            .map_err(|e| {
                error!("question add err: {:?}", e);
                Error::new(ErrorKind::Other, "变式题关联失败")
            })?;
    }

    Ok(row.id)
}

// 题目基本信息, 基本够列表使用
fn to_base_resp(row: &Question) -> QuestionBaseResp {
    QuestionBaseResp {
        id: row.id,
        question_cate_id: row.question_cate_id,
        question_type_id: row.question_type_id,
        question_tag_ids: row.question_tag_ids.clone(),
        author_id: row.author_id,
        title: row.title.clone(),
        content_plain: Some(row.content_plain.clone()),
        comment: row.comment.clone(),
        difficulty_level: row.difficulty_level,
        images: row.images.clone(),
        options: row.options.clone(),
        options_layout: row.options_layout,
        status: row.status,
        approve_id: row.approve_id,
        reject_reason: row.reject_reason.clone(),
        approve_at: row.approve_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

// 额外扩展信息
fn to_extra_resp(row: Question) -> QuestionExtraInfo {
    QuestionExtraInfo {
        answer: row.answer,
        knowledge: row.knowledge,
        analysis: row.analysis,
        process: row.process,
        remark: row.remark,
    }
}

// 完整的题目信息
fn to_info_resp(row: Question) -> QuestionInfoResp {
    QuestionInfoResp {
        base_info: to_base_resp(&row),
        extra_info: to_extra_resp(row),
    }
}

// 通过主键获取详情
pub async fn info(app_conf: web::Data<AppConfig>, id: i64) -> Result<QuestionInfoResp, Error> {
    let row = Question::find_by_id(&app_conf.get_ref().db, id)
        .await
        .map_err(|err| {
            error!("question get by id err: {:?}", err);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    Ok(to_info_resp(row))
}

// 题目列表
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: QuestionListReq,
) -> Result<QuestionListResp, Error> {
    let db = &app_conf.db; // 假设 AppConfig 暴露了 db 字段

    let status: i16 = req.status.unwrap_or(QuestionStatus::Published as i16);

    // 1. 查询总数
    let total = Question::count_by_cate_and_type(
        db,
        req.question_cate_id,
        status,
        req.question_type_id,
        req.ids.clone(),
        req.title_val.clone(),
        req.tag_ids.clone(),
    )
    .await
    .map_err(|e| {
        error!("question count by id err: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败") // 注意：这里直接返回 Error，不需要包裹 Err()
    })?;
    
    if total == 0 {
        return Ok(QuestionListResp {
            list: vec![],
            page_no: 0,
            page_size: 0,
            total,
        });
    }

    // 2. 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;

    // 3. 查询列表 (添加 ? 运算符解包 Result)
    let list_data = Question::list_by_cate_and_type(
        db,
        req.question_cate_id,
        status,
        req.question_type_id,
        req.ids,
        req.title_val,
        req.tag_ids,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("question list by id err: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?; // 必须加 ? 才能得到 Vec<Question>

    // 4. 转换并返回
    Ok(QuestionListResp {
        // 使用 map().collect() 一行转换
        list: list_data
            .into_iter()
            .map(|row| to_base_resp(&row))
            .collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}
