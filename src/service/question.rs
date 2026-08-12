use crate::AppConfig;
use crate::api::question::{
    CreateQuestionReq, DeleteReq, OriginalReq, QuestionBaseResp, QuestionExtraInfo,
    QuestionInfoResp, QuestionListReq, QuestionListResp, QuestionSimilarListReq,
};
use crate::r#enum::question::QuestionPageSource;
use crate::middleware::user::UserInfo;
use crate::model::question::{Question, QuestionStatus};
use crate::model::question_similar::{QuestionSimilar, QuestionSimilarType};
use crate::service::user::{get_user_map, get_user_name};
use crate::util::local::to_local_datetime;
use actix_web::web;
use log::error;
use regex::Regex;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

/// 将包含 LaTeX 的富文本标题转换为纯文本
pub fn to_plain_text(title: &str) -> String {
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
pub async fn add(
    app_conf: web::Data<AppConfig>,
    mut req: CreateQuestionReq,
    user_info: UserInfo,
) -> Result<i64, Error> {
    // 关于重复添加的问题应该要使用 redis 全局锁, 暂时没有 缓存服务
    let db = &app_conf.get_ref().db;

    let source_id = req.source_id;
    let is_add = req.id.is_none();

    // 题目上传只能上传草稿中和待审核的题目
    if !(req.status == QuestionStatus::Draft.as_i16()
        || req.status == QuestionStatus::Pending.as_i16())
    {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "不被允许的题目上传操作",
        ));
    }

    // 只允许编辑自己的题目
    if let Some(id) = req.id {
        let has_question = Question::find_by_id(db, id).await.map_err(|err| {
            error!("Failed to find question: {}", err);
            Error::new(ErrorKind::Other, "题目查询错误")
        })?;
        if has_question.author_id != user_info.user_id {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "只允许编辑自己的题目",
            ));
        }
    }

    // 从请求信息中解析出题目归属类型
    let question_similar_type =
        if let Some(question_similar_type) = req.question_similar_type.clone() {
            question_similar_type
        } else {
            QuestionSimilarType::Similar.as_i16()
        };

    // 从登录信息中解析出作者
    req.author_id = Some(user_info.user_id);

    req.content_plain = Some(to_plain_text(req.title.as_str()));

    let id = Question::simple_save(db, req).await.map_err(|e| {
        error!("question add err: {:?}", e);
        Error::new(ErrorKind::Other, "题目添加失败")
    })?;

    // 新增如果存在变式题则关联变式题
    if is_add && source_id.is_some() {
        let _ = QuestionSimilar::insert(db, source_id.unwrap(), id, question_similar_type)
            .await
            .map_err(|e| {
                error!("question add err: {:?}", e);
                Error::new(ErrorKind::Other, "变式题关联失败")
            })?;
    }

    Ok(id)
}

// 题目基本信息, 基本够列表使用
fn to_base_resp(row: &Question, author_name: String) -> QuestionBaseResp {
    QuestionBaseResp {
        id: row.id,
        question_cate_id: row.question_cate_id,
        question_type_id: row.question_type_id,
        question_tag_ids: row.question_tag_ids.clone(),
        question_dimension_ids: row.question_dimension_ids.clone(),
        author_id: row.author_id,
        author_name,
        source: row.source.clone(),
        original_name: row.original_name.clone(),
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
        approve_at: if let Some(at) = row.approve_at {
            Some(to_local_datetime(at))
        } else {
            None
        },
        steps: row.steps.clone(),
        created_at: to_local_datetime(row.created_at),
        updated_at: to_local_datetime(row.updated_at),
    }
}

// 额外扩展信息
fn to_extra_resp(row: &Question) -> QuestionExtraInfo {
    QuestionExtraInfo {
        answer: row.answer.clone(),
        knowledge: row.knowledge.clone(),
        analysis: row.analysis.clone(),
        process: row.process.clone(),
        remark: row.remark.clone(),
    }
}

// 完整的题目信息
pub fn to_info_resp(row: &Question, author_name: String) -> QuestionInfoResp {
    QuestionInfoResp {
        base_info: to_base_resp(row, author_name),
        extra_info: to_extra_resp(row),
    }
}

// 通过主键获取详情
pub async fn info(app_conf: web::Data<AppConfig>, id: i64) -> Result<QuestionInfoResp, Error> {
    let db = &app_conf.get_ref().db;
    let row = Question::find_by_id(db, id).await.map_err(|err| {
        error!("question get by id err: {:?}", err);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    let author_name = get_user_name(db, row.author_id).await;

    Ok(to_info_resp(&row, author_name))
}

// 题目列表
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: QuestionListReq,
    user_info: Option<UserInfo>,
) -> Result<QuestionListResp, Error> {
    let db = &app_conf.db;

    // 页面来源
    let req_source = QuestionPageSource::from_str(&req.source)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "不清楚的查询来源"))?;

    // 我的题目等时需要登录
    let (author_id, status) = if req_source == QuestionPageSource::List {
        (None, QuestionStatus::Published.as_i16())
    } else {
        let user_info =
            user_info.ok_or_else(|| Error::new(ErrorKind::PermissionDenied, "需要登录方能访问"))?;
        let status = req.status.unwrap_or(QuestionStatus::Published.as_i16());

        // 目前我的题目需要指定作者, 其它情况下暂时不需要指定作者
        if req_source == QuestionPageSource::MyQuestion {
            (Some(user_info.user_id), status)
        } else {
            (None, status)
        }
    };

    // 查询总数
    let total = Question::count_by_cate_and_type(
        db,
        req.question_cate_ids.clone(),
        status,
        req.question_type_id,
        req.ids.clone(),
        req.title_val.clone(),
        req.tag_ids.clone(),
        req.dimension_ids.clone(),
        author_id,
    )
    .await
    .map_err(|e| {
        error!("question count by id err: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    if total == 0 {
        return Ok(QuestionListResp {
            list: vec![],
            page_no: 1,
            page_size: 10,
            total,
        });
    }

    // 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;

    // 查询列表 (添加 ? 运算符解包 Result)
    let list_data = Question::list_by_cate_and_type(
        db,
        req.question_cate_ids,
        status,
        req.question_type_id,
        req.ids,
        req.title_val,
        req.tag_ids,
        req.dimension_ids,
        author_id,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("question list by id err: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    // 批量获取作者名称
    let author_ids: Vec<i64> = list_data.iter().map(|q| q.author_id).collect();
    let user_map: HashMap<i64, String> = get_user_map(db, author_ids).await?;

    // 转换并返回
    Ok(to_list_resp(
        list_data,
        user_map,
        req.page_no,
        req.page_size,
        total,
    ))
}

fn to_list_resp(
    list_data: Vec<Question>,
    user_map: HashMap<i64, String>,
    page_no: i32,
    page_size: i32,
    total: i64,
) -> QuestionListResp {
    QuestionListResp {
        // 使用 map().collect() 一行转换
        list: list_data
            .into_iter()
            .map(|row| {
                to_base_resp(
                    &row,
                    user_map
                        .get(&row.author_id)
                        .cloned()
                        .unwrap_or_else(|| "未知".to_string()),
                )
            })
            .collect(),
        page_no,
        page_size,
        total,
    }
}

// 变式题题目列表
pub async fn similar(
    app_conf: web::Data<AppConfig>,
    req: QuestionSimilarListReq,
) -> Result<QuestionListResp, Error> {
    let db = &app_conf.db;

    let status: i16 = req.status.unwrap_or(QuestionStatus::Published as i16);

    // 查询总数
    let total = Question::count_similar_by_params(
        db,
        req.question_id,
        status,
        req.question_cate_id,
        req.question_type_id,
        req.tag_ids.clone(),
        req.question_dimension_ids.clone(),
    )
    .await
    .map_err(|e| {
        error!("question similar count by id err: {:?}", e);
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

    // 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;

    // 查询列表 (添加 ? 运算符解包 Result)
    let list_data = Question::list_similar_by_params(
        db,
        req.question_id,
        status,
        req.question_cate_id,
        req.question_type_id,
        req.tag_ids,
        req.question_dimension_ids,
        req.page_size,
        offset,
    )
    .await
    .map_err(|e| {
        error!("question similar list by id err: {:?}", e);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    let author_ids: Vec<i64> = list_data.iter().map(|q| q.author_id).collect();
    let user_map: HashMap<i64, String> = get_user_map(db, author_ids).await?;

    // 转换并返回
    Ok(to_list_resp(
        list_data,
        user_map,
        req.page_no,
        req.page_size,
        total,
    ))
}

// 课本原题
pub async fn original(
    app_conf: web::Data<AppConfig>,
    req: OriginalReq,
) -> Result<Option<i64>, Error> {
    let id = Question::original(&app_conf.db, req.id)
        .await
        .map_err(|e| {
            error!("original request by id err: {:?}", e);
            Error::new(ErrorKind::Other, "题目查询错误")
        })?;

    Ok(id)
}

// 删除题目
pub async fn delete(
    app_conf: web::Data<AppConfig>,
    req: DeleteReq,
    user_info: UserInfo,
) -> Result<bool, Error> {
    if req.id <= 0 {
        return Err(Error::new(ErrorKind::Other, "题目标识为空"));
    }

    let db = &app_conf.db;

    // 只允许删除自己的题目
    let has_question = Question::find_by_id(db, req.id).await.map_err(|err| {
        error!("Failed to find question: {}", err);
        Error::new(ErrorKind::Other, "题目查询错误")
    })?;
    if has_question.author_id != user_info.user_id {
        return Err(Error::new(ErrorKind::Other, "只允许删除自己的题目"));
    }

    let rows = Question::delete(db, req.id).await.map_err(|err| {
        error!("question delete by id err: {:?}", err);
        Error::new(ErrorKind::Other, "查询失败")
    })?;

    Ok(rows > 0)
}
