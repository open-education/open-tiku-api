use crate::api::req::question::CreateQuestionReq;
use crate::api::req::text::QuestionSnippetReq;
use crate::app::conf::AppState;
use crate::constant::meta;
use crate::enums::question::{QuestionRelationType, QuestionStatus};
use crate::enums::task::{TaskStatus, TaskType};
use crate::model::other_dict::TextbookDict;
use crate::model::question::{Content, Question, QuestionOption};
use crate::model::question_relation::QuestionRelation;
use crate::model::task::Task;
use crate::service::question;
use crate::util::error::AppError;
use crate::util::markdown;
use crate::util::markdown::RawQuestion;
use rust_decimal::Decimal;
use sqlx::types::Json;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use tracing::{error, info};

// 批量题目上传
pub async fn batch(app_state: &AppState) -> Result<(), AppError> {
    let db = &app_state.db;

    // 查询所有待执行的任务
    let waiting_task_list = Task::get_waiting_list(db, TaskType::UploadQuestion as i16)
        .await
        .map_err(|e| {
            error!("Get waiting task list err: {}", e);
            AppError::db_error("待运行的任务列表查询失败")
        })?;
    if waiting_task_list.is_empty() {
        info!("Waiting task list is empty");
        return Ok(());
    }

    // 得到所有的教材通用字典
    let mut textbook_ids: Vec<i32> = waiting_task_list
        .iter()
        .map(|item| item.textbook_id)
        .collect();
    textbook_ids.sort_unstable();
    textbook_ids.dedup();
    let rows = TextbookDict::find_by_textbook_ids(db, &textbook_ids, None)
        .await
        .map_err(|e| {
            error!("find textbook dict err: {}", e);
            AppError::db_error("查询教材字典出错")
        })?;

    let mut map: HashMap<i32, HashMap<String, Vec<TextbookDict>>> = HashMap::new();
    for item in rows {
        map.entry(item.textbook_id)
            .or_default()
            .entry(item.type_code.clone())
            .or_default()
            .push(item);
    }

    // 遍历待处理的任务列表
    for task_info in waiting_task_list {
        // 修改任务为执行中, 后续的失败不回滚, 而是继续更新状态
        // 如果文件内容本身正常只是程序问题导致, 后续需要手动修改为待执行等待下一批次重新执行
        // 先更新状态为 Running（可允许失败时跳过该任务）
        let task_id = task_info.id;
        let task_name = task_info.name.clone();
        if let Err(e) =
            Task::update_by_id(db, &task_id, TaskStatus::Running as i16, String::new()).await
        {
            error!(
                "Update task id: {}, name {} status=Running err: {}",
                task_id, task_name, e
            );
            // 不 return，继续下一个任务
            continue;
        }

        // 处理一个任务, 一个任务的事务是独立的
        let task_name = task_info.name.clone();
        info!("Process single task info process start: {}", task_name);
        if let Err(e) = single(app_state, task_info, &map).await {
            error!("Process single task info err: {}", e.msg);
            // 更新当前任务执行失败, 数据库记录原因为捕获的错误信息, 实际的执行内容需要看脚本执行日志
            if let Err(e) = Task::update_by_id(db, &task_id, TaskStatus::Failed as i16, e.msg).await
            {
                error!(
                    "Update task id: {}, name {} status=Failed err: {}",
                    task_id, task_name, e
                );
                // 不 return，继续下一个任务
                continue;
            }
            continue;
        }
        info!("Process single task info process done: {}", task_name);
    }

    info!("Waiting task list all done");

    Ok(())
}

// 上传单个题目文件
async fn single(
    app_state: &AppState,
    task_info: Task,
    map: &HashMap<i32, HashMap<String, Vec<TextbookDict>>>,
) -> Result<(), AppError> {
    // 记录结果日志
    let mut result: Vec<String> = vec![];

    // 读取文件内容 url 字段存取的是文件名称, 路径需要系统设计补完整
    let file_path = format!(
        "{}/{}/{}",
        app_state.config.meta.path,
        meta::FILE_NAME,
        task_info.url
    );
    let content = fs::read_to_string(file_path.as_str()).map_err(|err| {
        error!("read file {} err: {}", file_path, err);
        AppError::internal_error("读取文件内容失败")
    })?;

    result.push("读取文件".to_string());

    let all_questions = markdown::get_questions(&content)?;
    if all_questions.is_empty() {
        error!("Task name: {} all questions is empty", task_info.name);
        return Err(AppError::business_error("该文件没有读取到任何有效的题目"));
    }

    // 这部分更新使用事务
    let mut tx = app_state.db.begin().await.map_err(|e| {
        error!("Error beginning transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    // 一个文件作为一个事务单位
    for question_info in all_questions {
        // 母题分层体系
        let parent_level = question_info.parent.level.clone();
        result.push(format!("添加 {}", parent_level));

        let parent_req = to_req(
            question_info.parent,
            None,
            QuestionRelationType::Base,
            &task_info,
            map,
        )?;
        info!("Add parent question name: {} begin", parent_level);
        // 母题标题
        let p_title = parent_req.title.clone();
        let parent = Question::tx_insert(&mut tx, parent_req)
            .await
            .map_err(|err| {
                error!("Insert parent of question err: {}", err);
                AppError::db_error("母题添加失败")
            })?;
        result.push(format!("添加 {}", p_title));

        // 变式题列表为空正常
        if question_info.children.is_empty() {
            continue;
        }

        let mut children_req: Vec<CreateQuestionReq> = vec![];
        for child in question_info.children {
            let child_level = child.level.clone();
            info!("Add child question name: {} begin", child_level);
            result.push(format!("添加 {}", child_level));

            let child_req = to_req(
                child,
                Some(parent.id),
                QuestionRelationType::Similar,
                &task_info,
                map,
            )?;

            // 子题标题
            let c_title = child_req.title.clone();
            children_req.push(child_req);

            result.push(format!("添加 {}", c_title));
        }

        // 得到所有添加的变式题主键列表
        let children_ids = Question::tx_batch_insert(&mut tx, children_req)
            .await
            .map_err(|err| {
                error!("Batch insert child of question err: {}", err);
                AppError::db_error("批量添加变式题失败")
            })?;
        info!("Add all child question end");
        result.push("变式题添加完成".to_string());

        info!("Add relation parent child question begin");
        let similar_pairs: Vec<(i64, i64, i16)> = children_ids
            .into_iter()
            .map(|child| (parent.id, child, QuestionRelationType::Similar as i16))
            .collect();

        // 关联母题和变式题对应关系
        QuestionRelation::batch_insert(&mut tx, similar_pairs)
            .await
            .map_err(|e| {
                error!("Batch insert child of question similar relation err: {}", e);
                AppError::db_error("母题和变式题关联失败")
            })?;
        info!("Add relation parent child question end");

        result.push("关联母题和变式题完成".to_string());

        info!("Add parent question name: {} end", parent_level);
    }

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    result.push("文件处理完成".to_string());

    // 更新任务列表为执行成功
    if let Err(e) = Task::update_by_id(
        &app_state.db,
        &task_info.id,
        TaskStatus::Success as i16,
        result.join("\n"),
    )
    .await
    {
        error!(
            "Task done, but update task id: {}, name {} status=Failed err: {}",
            task_info.id, task_info.name, e
        );
        // 这次更新失败不做任何处理, 需要关注这类日志
    }

    Ok(())
}

// 根据题目类型列表获取对应的题目类型标识和选项内容
fn get_question_type_and_options(
    question_type: &str,
    choices: &[(char, String)],
    map: &HashMap<String, Vec<TextbookDict>>,
) -> Result<(i32, Option<Json<Vec<QuestionOption>>>), AppError> {
    let type_list: &[TextbookDict] = map
        .get("question_type")
        .ok_or_else(|| AppError::not_found("题目类型字典为空"))?;

    // 查找匹配的题型记录：优先包含匹配，否则取第一个非选择题
    let question_type_info = type_list
        .iter()
        .find(|item| item.item_value.contains(question_type))
        .or_else(|| type_list.iter().find(|item| !item.is_select));

    // 获取题型 ID
    let question_type_id = question_type_info
        .map(|item| item.id.unwrap_or_default())
        .ok_or_else(|| AppError::not_found("题目类型无法匹配到选项字典"))?;

    // 处理选择题选项
    let options = if let Some(info) = question_type_info {
        if info.is_select {
            let opts: Vec<QuestionOption> = choices
                .iter()
                .enumerate()
                .map(|(idx, (label, content))| QuestionOption {
                    label: label.to_string(),
                    content: content.clone(),
                    images: None,
                    order: (idx + 1) as i32,
                })
                .collect();
            Some(Json(opts))
        } else {
            None
        }
    } else {
        None
    };

    Ok((question_type_id, options))
}

fn get_tag_ids(
    parent_id: Option<i64>,
    map: &HashMap<String, Vec<TextbookDict>>,
) -> Result<Option<Vec<i32>>, AppError> {
    let tag_list: &[TextbookDict] = map
        .get("question_tag")
        .ok_or_else(|| AppError::not_found("题目标签字典为空"))?;
    let question_tag_info = tag_list
        .iter()
        .find(|item| item.item_value.contains("变式题"));
    let question_tag_ids: Option<Vec<i32>> = if parent_id.is_some() {
        question_tag_info.map(|tag_info| vec![tag_info.id.unwrap_or_default()])
    } else {
        None
    };

    Ok(question_tag_ids)
}

fn get_level_id(level: &str, map: &HashMap<String, Vec<TextbookDict>>) -> Result<i32, AppError> {
    let level_list: &[TextbookDict] = map
        .get("question_level")
        .ok_or_else(|| AppError::not_found("分层体系字典为空"))?;

    let matched_item = level_list
        .iter()
        .find(|item| level.starts_with(item.item_value.as_str()))
        .ok_or_else(|| AppError::not_found("未匹配到对应的分层体系"))?;

    Ok(matched_item.id.unwrap_or_default())
}

fn get_dict_ids(
    req_list: &[String],
    type_code: &str,
    map: &HashMap<String, Vec<TextbookDict>>,
    err_msg: &str,
) -> Result<Vec<i32>, AppError> {
    let list: &[TextbookDict] = map
        .get(type_code)
        .ok_or_else(|| AppError::not_found(err_msg))?;
    let ids: Vec<i32> = list
        .iter()
        .filter(|item| req_list.iter().any(|val| item.item_value.contains(val)))
        .map(|item| item.id.unwrap_or_default())
        .collect();

    Ok(ids)
}

// 解析出题目难度, 解析失败等均返回 1
fn get_difficulty_level(val: &str) -> Decimal {
    // 允许的分数集合使用 Decimal
    const ALLOWED: [&str; 9] = ["1", "1.5", "2", "2.5", "3", "3.5", "4", "4.5", "5"];

    // 解析为 Decimal
    let num = Decimal::from_str(val.trim()).unwrap_or_else(|_| Decimal::from(1));

    // 检查是否在允许列表中（通过字符串比较或转为字符串后比较）
    let num_str = num.to_string();
    if ALLOWED.contains(&num_str.as_str()) {
        num
    } else {
        Decimal::from(1)
    }
}

// 通过 markdown 文档文本内容转为请求体
fn to_req(
    raw: RawQuestion,
    parent_id: Option<i64>,
    relation_type: QuestionRelationType,
    task_info: &Task,
    map: &HashMap<i32, HashMap<String, Vec<TextbookDict>>>,
) -> Result<CreateQuestionReq, AppError> {
    let dict_map = map.get(&task_info.textbook_id).ok_or_else(|| {
        error!("textbook_id: {} dict is empty", task_info.textbook_id);
        AppError::not_found("教材通用字典不存在")
    })?;

    // 题目类型
    let (question_type_id, options) =
        get_question_type_and_options(&raw.question_type, &raw.choices, dict_map)?;
    if question_type_id <= 0 {
        return Err(AppError::business_error("解析后无法匹配上题目类型"));
    }

    // 题目标签
    let question_tag_ids = get_tag_ids(parent_id, dict_map)?;

    // 核心素养
    let dimension_ids: Vec<i32> = get_dict_ids(
        &raw.dimensions,
        "question_dimension",
        dict_map,
        "核心素养字典为空",
    )?;

    // 适用场景
    let scene_ids: Vec<i32> =
        get_dict_ids(&raw.scenes, "question_scene", dict_map, "适用场景字典为空")?;

    // 常见错误
    let mistake_tip_ids: Vec<i32> = get_dict_ids(
        &raw.mistake_tips,
        "question_mistake_tip",
        dict_map,
        "常见错误字典为空",
    )?;

    let req = CreateQuestionReq {
        id: None,
        question_cate_id: task_info.question_cate_id as i32,
        source_id: parent_id,
        relation_type: relation_type as i16,
        question_type_id,
        question_tag_ids,
        question_dimension_ids: Some(dimension_ids),
        level_id: get_level_id(&raw.level, dict_map)?,
        scene_ids: Some(scene_ids),
        mistake_tip_ids: Some(mistake_tip_ids),
        author_id: Some(task_info.author_id),
        source: String::from("题目上传"),
        original_name: String::new(),
        status: QuestionStatus::Draft as i16,
        title: raw.title.clone(),
        content_plain: Some(question::to_plain_text(&raw.title)),
        comment: None,
        difficulty_level: get_difficulty_level(&raw.difficulty_level),
        images: None,
        options,
        options_layout: Some(1),
        answer: Some(raw.answer),
        knowledge: Some(raw.knowledge.join(", ")),
        analysis: Some(Json(Content {
            content: raw.analysis,
            images: None,
        })),
        process: Some(Json(Content {
            content: raw.detail,
            images: None,
        })),
        steps: None,
        remark: None,
        remark_ext: Some(String::from("批量题目上传")),
    };

    Ok(req)
}

// 从markdown片段文本中解析出题目信息
pub async fn parse_question_snippet(
    app_state: &AppState,
    req: QuestionSnippetReq,
) -> Result<CreateQuestionReq, AppError> {
    if req.textbook_id <= 0 {
        return Err(AppError::param_error("教材标识不能为空"));
    }
    if req.content.is_empty() {
        return Err(AppError::param_error("接收内容不能为空"));
    }

    let raw = markdown::get_question(req.content.as_str())?;
    if raw.title.is_empty() {
        return Err(AppError::business_error("解析后无法查找到题目题干"));
    }

    // 获取通用字典
    let db = &app_state.db;

    let codes: Vec<String> = vec![
        "question_type".to_string(),
        "question_tag".to_string(),
        "question_dimension".to_string(),
        "question_scene".to_string(),
        "question_mistake_tip".to_string(),
    ];
    let rows = TextbookDict::find_by_textbook_ids(db, &[req.textbook_id], Some(codes))
        .await
        .map_err(|e| {
            error!("error finding unique textbook item: {}", e);
            AppError::db_error("查询教材通用字典出错")
        })?;

    let mut map: HashMap<String, Vec<TextbookDict>> = HashMap::new();
    for item in rows {
        map.entry(item.type_code.clone()).or_default().push(item);
    }

    // 题目类型
    let (question_type_id, options) =
        get_question_type_and_options(&raw.question_type, &raw.choices, &map)?;
    if question_type_id <= 0 {
        return Err(AppError::business_error("解析后无法匹配上题目类型"));
    }

    // 核心素养
    let dimension_ids: Vec<i32> = get_dict_ids(
        &raw.dimensions,
        "question_dimension",
        &map,
        "核心素养字典为空",
    )?;

    // 适用场景
    let scene_ids: Vec<i32> =
        get_dict_ids(&raw.scenes, "question_scene", &map, "适用场景字典为空")?;

    // 常见错误
    let mistake_tip_ids: Vec<i32> = get_dict_ids(
        &raw.mistake_tips,
        "question_mistake_tip",
        &map,
        "常见错误字典为空",
    )?;

    Ok(CreateQuestionReq {
        id: None,
        question_cate_id: 0,
        source_id: None,
        relation_type: QuestionRelationType::Base as i16,
        question_type_id,
        question_tag_ids: None,
        question_dimension_ids: Some(dimension_ids),
        level_id: 0,
        scene_ids: Some(scene_ids),
        mistake_tip_ids: Some(mistake_tip_ids),
        author_id: None,
        source: String::new(),
        original_name: String::new(),
        status: QuestionStatus::Draft as i16,
        title: raw.title.clone(),
        content_plain: Some(question::to_plain_text(&raw.title)),
        comment: None,
        difficulty_level: get_difficulty_level(&raw.difficulty_level),
        images: None,
        options,
        options_layout: Some(1),
        answer: Some(raw.answer),
        knowledge: Some(raw.knowledge.join(", ")),
        analysis: Some(Json(Content {
            content: raw.analysis,
            images: None,
        })),
        process: Some(Json(Content {
            content: raw.detail,
            images: None,
        })),
        steps: None,
        remark: None,
        remark_ext: Some(String::from("文本片段解析")),
    })
}
