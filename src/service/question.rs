use crate::api::req::question::{
    CreateQuestionReq, DeleteReq, QuestionListReq, QuestionSimilarListReq, UndoReq,
};
use crate::api::resp::question::{
    CountQuestionResp, QuestionBaseResp, QuestionInfoResp, QuestionListResp,
};
use crate::app::conf::WebAppState;
use crate::enums::question::{QuestionPageSource, QuestionRelationType, QuestionStatus};
use crate::middleware::user::UserInfo;
use crate::model::question::{
    CateAndTypeReq, Question, SimilarReq, TextbookPathInfo, UpdateQuestionReq,
};
use crate::model::question_relation::QuestionRelation;
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use regex::Regex;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tracing::error;

// 将包含 LaTeX 的富文本标题转换为纯文本
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
    app_state: &WebAppState,
    mut req: CreateQuestionReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    // 关于重复添加的问题应该要使用 redis 全局锁, 暂时没有 缓存服务
    let db = &app_state.db;

    let source_id = req.source_id.unwrap_or_default();
    let req_id = req.id.unwrap_or_default();
    let is_add = req.id.is_none();

    // 题目上传只能上传草稿中的题目
    if req.status != QuestionStatus::Draft as i16 {
        return Err(AppError::param_error("仅支持添加草稿中的题目"));
    }

    // 从请求信息中解析出题目归属类型
    let relation_type = QuestionRelationType::from_i16(req.relation_type)
        .ok_or_else(|| AppError::param_error("非法的题目类型"))?;

    if relation_type == QuestionRelationType::Base && source_id > 0 {
        return Err(AppError::param_error("参数冲突: 母题不支持设置来源标识"));
    }

    // 只允许编辑自己的题目, 实际上题目应该可以公开编辑, 但是这个要引入版本控制, 即记录谁做了什么
    if req_id > 0 {
        let has_question = Question::find_by_id(db, req_id).await.map_err(|err| {
            error!("Failed to find question (id: {req_id}): {err}");
            AppError::db_error("题目查询失败")
        })?;

        if has_question.author_id != user_info.user_id {
            return Err(AppError::permission_denied(
                "操作拒绝：只允许编辑自己创建的题目",
            ));
        }
    }

    // 填充章节和知识点路径
    let path_info = get_path_info_by_cate_id(db, req.question_cate_id).await?;
    req.chapter_id = Some(path_info.c_id);
    req.chapter_path = Some(format!("{}/{}/", path_info.c_path, path_info.c_id));
    req.knowledge_id = Some(path_info.k_id);
    req.knowledge_path = Some(format!("{}/{}/", path_info.k_path, path_info.k_id));

    // 如果是新增课本原题, 母题只能关联一个课本原题, 编辑它自己不需要处理
    if is_add && relation_type == QuestionRelationType::Original {
        // 来源题目必须是母题
        if source_id == 0 {
            return Err(AppError::param_error(
                "缺失必要参数: 课本原题必须指定来源母题 ID",
            ));
        }

        // 检查是否是变式题
        let similar = QuestionRelation::find_base_by_similar_id(db, source_id)
            .await
            .map_err(|err| {
                error!("Failed to find similar by child_id ({source_id}): {err}");
                AppError::db_error("变式题关系查询失败")
            })?;

        if similar.is_some() {
            return Err(AppError::business_error(
                "关系冲突：只有母题可以关联课本原题",
            ));
        }

        let child_ids = QuestionRelation::find_original_by_base_id(db, req_id)
            .await
            .map_err(|err| {
                error!("Failed to find original child ids (req_id: {req_id}): {err}");
                AppError::db_error("母题查找变式题失败")
            })?;
        if child_ids.len() > 1 {
            return Err(AppError::business_error(
                "重复关联, 母题只能关联一道课本原题",
            ));
        }
    }

    // 从登录信息中解析出作者
    req.author_id = Some(user_info.user_id);

    req.content_plain = Some(to_plain_text(req.title.as_str()));

    let id = Question::simple_save(db, req).await.map_err(|e| {
        error!("question add err: {e}");
        AppError::db_error("题目添加失败")
    })?;

    // 新增如果存在变式题则关联变式题
    if is_add && source_id > 0 {
        let _ = QuestionRelation::insert(db, source_id, id, relation_type as i16)
            .await
            .map_err(|e| {
                error!("question add err: {e}");
                AppError::db_error("题目关联关系关联失败")
            })?;
    }

    Ok(id)
}

// 填充章节和知识点信息
pub async fn get_path_info_by_cate_id(
    pool: &PgPool,
    cate_id: i32,
) -> Result<TextbookPathInfo, AppError> {
    // 获取章节和知识点父级路径信息
    let mut path_info_rows = Question::find_path_info_by_cate_id(pool, cate_id)
        .await
        .map_err(|err| {
            error!("Question add find textbook path info err: {err}");
            AppError::db_error("查询题型父路径信息出错")
        })?;
    let path_info = match path_info_rows.pop() {
        Some(info) if path_info_rows.is_empty() => info, // 弹出1个后变空，说明严格等于1条
        None => return Err(AppError::business_error("题型关联信息不存在")),
        Some(_) => {
            return Err(AppError::business_error(
                "题型关联的父路径信息维护异常(存在多条记录)",
            ));
        }
    };

    Ok(path_info)
}

// 通过主键获取详情
pub async fn info(app_state: &WebAppState, id: i64) -> Result<QuestionInfoResp, AppError> {
    let db = &app_state.db;
    let row = Question::find_by_id(db, id).await.map_err(|err| {
        error!("question get by id err: {err}");
        AppError::db_error("查询失败")
    })?;

    let user_name_map: HashMap<i64, String> = {
        let mut user_ids = Vec::with_capacity(2);
        user_ids.push(row.author_id);
        if row.approve_id > 0 && row.approve_id != row.author_id {
            user_ids.push(row.approve_id);
        }
        get_user_map(db, user_ids).await?
    };

    let author_name = user_name_map
        .get(&row.author_id)
        .cloned()
        .unwrap_or_default();
    let approve_name = user_name_map
        .get(&row.approve_id)
        .cloned()
        .unwrap_or_default();

    let resp = (row, author_name, approve_name).into();

    Ok(resp)
}

// 题目列表
pub async fn list(
    app_state: &WebAppState,
    req: QuestionListReq,
    user_info: Option<UserInfo>,
) -> Result<QuestionListResp, AppError> {
    let db = &app_state.db;

    // 页面来源
    let req_source = QuestionPageSource::from_str(&req.source)
        .ok_or_else(|| AppError::param_error("不清楚的查询来源"))?;

    // 章节或知识点不能同时为空
    check_chapter_knowledge_is_full(&req.chapter_path, &req.knowledge_path)?;

    // 我的题目等时需要登录
    let (author_id, status) = if req_source == QuestionPageSource::List {
        (None, Some(QuestionStatus::Published as i16))
    } else {
        let user_info = user_info.ok_or_else(|| AppError::permission_denied("需要登录方能访问"))?;
        // 目前我的题目需要指定作者, 其它情况下暂时不需要指定作者, 后续审核查询条件重新指定
        // 我的题目和审核状态由前端自己指定
        if req_source == QuestionPageSource::MyQuestion {
            (Some(user_info.user_id), req.status)
        } else {
            (None, req.status)
        }
    };

    let query_req: CateAndTypeReq = CateAndTypeReq {
        chapter_path: req.chapter_path,
        knowledge_path: req.knowledge_path,
        cate_ids: req.question_cate_ids,
        status,
        type_id: req.question_type_id,
        ids: req.ids,
        title_val: req.title_val,
        tag_ids: req.tag_ids,
        dimension_ids: req.dimension_ids,
        author_id,
        level_ids: req.level_ids,
        scene_ids: req.scene_ids,
        mistake_tip_ids: req.mistake_tip_ids,
    };

    // 查询总数
    let total = Question::count_by_cate_and_type(db, &query_req)
        .await
        .map_err(|e| {
            error!("question count by id err: {e}");
            AppError::db_error("题目计数查询失败")
        })?;

    // 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(QuestionListResp {
            list: Vec::new(),
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    // 查询列表 (添加 ? 运算符解包 Result)
    let rows = Question::list_by_cate_and_type(db, &query_req, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("question list by id err: {e}");
            AppError::db_error("题目列表查询失败")
        })?;

    let mut user_ids_set = HashSet::with_capacity(rows.len() * 2);

    for q in &rows {
        user_ids_set.insert(q.author_id);
        if q.approve_id > 0 {
            user_ids_set.insert(q.approve_id);
        }
    }

    let user_ids: Vec<i64> = user_ids_set.into_iter().collect();
    let user_map: HashMap<i64, String> = get_user_map(db, user_ids).await?;

    Ok(QuestionListResp {
        list: to_list_resp(rows, &user_map),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

// 检查章节和知识点不能同时为空
pub fn check_chapter_knowledge_is_full(
    chapter_path: &Option<String>,
    knowledge_path: &Option<String>,
) -> Result<(), AppError> {
    let has_chapter = chapter_path
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());
    let has_knowledge = knowledge_path
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty());
    if !has_chapter && !has_knowledge {
        return Err(AppError::business_error("章节路径和知识点路径不能同时为空"));
    }

    Ok(())
}

fn to_list_resp(rows: Vec<Question>, user_map: &HashMap<i64, String>) -> Vec<QuestionBaseResp> {
    let mut list: Vec<QuestionBaseResp> = Vec::with_capacity(rows.len());
    for row in rows {
        let author_name = user_map.get(&row.author_id).cloned().unwrap_or_default();
        let approve_name = user_map.get(&row.approve_id).cloned().unwrap_or_default();
        let info: QuestionInfoResp = (row, author_name, approve_name).into();
        list.push(info.base_info);
    }

    list
}

// 变式题题目列表
pub async fn similar(
    app_state: &WebAppState,
    req: QuestionSimilarListReq,
) -> Result<QuestionListResp, AppError> {
    let db = &app_state.db;

    // 页面来源
    let req_source = QuestionPageSource::from_str(&req.source)
        .ok_or_else(|| AppError::param_error("不清楚的查询来源"))?;

    // 我的题目等不关注状态按前端为准
    let status = if req_source == QuestionPageSource::List {
        Some(QuestionStatus::Published as i16)
    } else {
        req.status
    };

    let query_req: SimilarReq = SimilarReq {
        question_id: req.question_id,
        status,
        cate_id: req.question_cate_id,
        type_id: req.question_type_id,
        tag_ids: req.tag_ids,
        dimension_ids: req.question_dimension_ids,
    };

    // 查询总数
    let total = Question::count_similar_by_params(db, &query_req)
        .await
        .map_err(|e| {
            error!("question similar count by id err: {e}");
            AppError::db_error("变式题计数查询失败")
        })?;

    // 计算偏移量
    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(QuestionListResp {
            list: Vec::new(),
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    let rows = Question::list_similar_by_params(db, &query_req, req.page_size, offset)
        .await
        .map_err(|e| {
            error!("question similar list by id err: {e}");
            AppError::db_error("变式题列表查询失败")
        })?;

    let author_ids: Vec<i64> = rows.iter().map(|q| q.author_id).collect();
    let user_map: HashMap<i64, String> = get_user_map(db, author_ids).await?;

    Ok(QuestionListResp {
        list: to_list_resp(rows, &user_map),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

// 删除题目
pub async fn delete(
    app_state: &WebAppState,
    req: DeleteReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("题目标识为空"));
    }

    let db = &app_state.db;

    // 只允许删除自己的题目
    let has_question = Question::find_by_id(db, req.id).await.map_err(|err| {
        error!("Failed to find question: {err}");
        AppError::db_error("题目查询错误")
    })?;
    if has_question.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只允许删除自己的题目"));
    }

    // 只有草稿的题目可以删除
    if has_question.status != QuestionStatus::Draft as i16 {
        return Err(AppError::business_error("只允许删除草稿中的题目"));
    }

    let rows = Question::delete_by_id(db, req.id).await.map_err(|err| {
        error!("question delete by id err: {err}");
        AppError::db_error("题目删除失败")
    })?;

    Ok(rows > 0)
}

pub async fn count(
    app_state: &WebAppState,
    user_info: UserInfo,
) -> Result<Vec<CountQuestionResp>, AppError> {
    let db = &app_state.db;

    let rows = Question::find_count_by_author_id(db, user_info.user_id)
        .await
        .map_err(|err| {
            error!("question count by id err: {err}");
            AppError::db_error("统计用户题目状态计数出错")
        })?;

    let result: Vec<CountQuestionResp> = rows.into_iter().map(Into::into).collect();

    Ok(result)
}

pub async fn undo(
    app_state: &WebAppState,
    req: UndoReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("题目标识不存在"));
    }

    let db = &app_state.db;

    let question = Question::find_by_id(db, req.id).await.map_err(|e| {
        error!("Select question id: {} err: {e}", req.id);
        AppError::db_error("查询题目失败")
    })?;
    if question.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只能撤回自己的题目"));
    }
    if question.status != QuestionStatus::Published as i16 {
        return Err(AppError::business_error("只有已发布的题目需要撤回"));
    }

    let update_req: UpdateQuestionReq = UpdateQuestionReq {
        id: req.id,
        status: QuestionStatus::Draft as i16,
        approve_id: user_info.user_id,
        reject_reason: Some(String::new()), // 撤回草稿要把之前的审核信息清空
        approve_at: None,
    };
    let rows_affected = Question::update_status_by_id(db, update_req)
        .await
        .map_err(|e| {
            error!("Update status by id: {} err: {e}", req.id);
            AppError::db_error("更新题目失败")
        })?;

    Ok(rows_affected > 0)
}
