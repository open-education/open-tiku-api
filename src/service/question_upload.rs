use crate::AppConfig;
use crate::api::question::CreateQuestionReq;
use crate::constant::meta;
use crate::model::other_dict::TextbookDict;
use crate::model::question::{Content, Question, QuestionOption};
use crate::model::question_similar::QuestionSimilar;
use crate::model::task::{Task, TaskStatus, TaskType};
use crate::service::question;
use crate::util::markdown_parse;
use log::{error, info, warn};
use sqlx::types::Json;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};

// 批量题目上传
pub async fn batch(app_conf: &AppConfig) -> Result<(), Error> {
    let db = &app_conf.db;

    // 查询所有待执行的任务
    let waiting_task_list = Task::get_waiting_list(db, TaskType::UploadQuestion as i16)
        .await
        .map_err(|e| {
            error!("Get waiting task list err: {}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;
    if waiting_task_list.is_empty() {
        info!("Waiting task list is empty");
        return Ok(());
    }

    // 缓存题型类型列表
    let mut question_type_cache: HashMap<i32, Vec<TextbookDict>> = HashMap::new();

    // 遍历待处理的任务列表
    for task_info in waiting_task_list {
        // 获取题型类型列表
        let textbook_id = task_info.textbook_id;

        let question_type_list = if let Some(list) = question_type_cache.get(&textbook_id) {
            list
        } else {
            // 未缓存，查询数据库（异步）
            let list =
                match TextbookDict::find_by_textbook_and_type(db, textbook_id, "question_type")
                    .await
                {
                    Ok(list) if !list.is_empty() => list,
                    Ok(_) => vec![], // 空结果也缓存
                    Err(e) => {
                        error!("查询 textbook_id {} 失败: {}", textbook_id, e);
                        vec![] // 失败也缓存空列表，避免反复尝试
                    }
                };
            question_type_cache.insert(textbook_id, list);
            question_type_cache.get(&textbook_id).unwrap()
        };

        // 修改任务为执行中, 后续的失败不回滚, 而是继续更新状态
        // 如果文件内容本身正常只是程序问题导致, 后续需要手动修改为待执行等待下一批次重新执行
        // 先更新状态为 Running（可允许失败时跳过该任务）
        let task_id = task_info.id;
        let task_name = task_info.name.clone();
        if let Err(e) =
            Task::update_by_id(db, &task_id, TaskStatus::Running as i16, "".to_string()).await
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
        if let Err(e) = single(app_conf, task_info, question_type_list).await {
            error!("Process single task info err: {}", e);
            // 更新当前任务执行失败, 数据库记录原因为捕获的错误信息, 实际的执行内容需要看脚本执行日志
            if let Err(e) =
                Task::update_by_id(db, &task_id, TaskStatus::Failed as i16, e.to_string()).await
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
    app_config: &AppConfig,
    task_info: Task,
    question_type_list: &Vec<TextbookDict>,
) -> Result<(), Error> {
    // 读取文件内容 url 字段存取的是文件名称, 路径需要系统设计补完整
    let file_path = format!(
        "{}/{}/{}",
        app_config.meta_path,
        meta::FILE_NAME,
        task_info.url
    );
    let content = fs::read_to_string(file_path)?;

    let all_questions = markdown_parse::get_questions(&content)?;
    if all_questions.is_empty() {
        warn!("Task name: {} all questions is empty", task_info.name);
        return Ok(());
    }

    // 这部分更新使用事务
    let mut tx = app_config.db.begin().await.map_err(|e| {
        error!("Error beginning transaction: {}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    // 一个文件作为一个事务单位
    for question_info in all_questions {
        // 母题
        let parent_req = to_req(question_info.parent, None, &task_info, &question_type_list);
        let simple_parent_title = parent_req.content_plain.clone().unwrap_or_default();
        info!("Add parent question name: {} begin", simple_parent_title);
        let parent = Question::tx_insert(&mut tx, parent_req)
            .await
            .map_err(|err| {
                error!("Insert parent of question err: {}", err);
                Error::new(ErrorKind::Other, "母题添加失败")
            })?;

        // 变式题列表
        if question_info.children.is_empty() {
            continue;
        }
        let mut children_req: Vec<CreateQuestionReq> = vec![];
        for child in question_info.children {
            let child_req = to_req(child, Some(parent.id), &task_info, &question_type_list);
            let simple_child_title = child_req.content_plain.clone().unwrap_or_default();
            info!("Add child question name: {} begin", simple_child_title);
            children_req.push(child_req);
        }

        // 得到所有添加的变式题主键列表
        let children_ids = Question::tx_batch_insert(&mut tx, children_req)
            .await
            .map_err(|err| {
                error!("Batch insert child of question err: {}", err);
                Error::new(ErrorKind::Other, "批量添加变式题失败")
            })?;
        info!("Add all child question end");
        if children_ids.is_empty() {
            continue;
        }

        info!("Add relation parent child question begin");
        let similar_pairs: Vec<(i64, i64)> = children_ids
            .into_iter()
            .map(|child| (parent.id, child))
            .collect();

        // 关联母题和变式题对应关系
        QuestionSimilar::batch_insert(&mut tx, similar_pairs)
            .await
            .map_err(|e| {
                error!("Batch insert child of question similar relation err: {}", e);
                Error::new(ErrorKind::Other, "母题和变式题关联失败")
            })?;
        info!("Add relation parent child question end");

        info!("Add parent question name: {} end", simple_parent_title);
    }

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        Error::new(ErrorKind::Other, "题目上传任务提交失败")
    })?;

    // 更新任务列表为执行成功
    if let Err(e) = Task::update_by_id(
        &app_config.db,
        &task_info.id,
        TaskStatus::Success as i16,
        "".to_string(),
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

// 通过 markdown 文档文本内容转为请求体
fn to_req(
    raw: markdown_parse::RawQuestion,
    parent_id: Option<i64>,
    task_info: &Task,
    question_type_list: &[TextbookDict],
) -> CreateQuestionReq {
    // 1. 获取题型字符串，默认填空
    let mut question_type_str = markdown_parse::get_question_type(&raw.table);
    if question_type_str.is_empty() {
        question_type_str = "填空题".to_string();
    }

    // todo 临时将选择题映射为 单选题
    if question_type_str.eq("选择题") || question_type_str.contains("选择") {
        question_type_str = "单选题".to_string();
    }

    // 2. 查找匹配的题型记录：优先包含匹配，否则取第一个非选择题
    let question_type_info = question_type_list
        .iter()
        .find(|item| item.item_value.contains(&question_type_str))
        .or_else(|| question_type_list.iter().find(|item| !item.is_select));

    // 3. 获取题型 ID，若未找到则 -1
    let question_type_id = question_type_info.map(|item| item.id).unwrap_or(-1);

    // 4. 处理选择题选项
    let options = if let Some(info) = question_type_info {
        if info.is_select {
            let choices = markdown_parse::get_choices(raw.choices);
            let opts: Vec<QuestionOption> = choices
                .into_iter()
                .enumerate()
                .map(|(idx, (label, content))| QuestionOption {
                    label: label.to_string(),
                    content,
                    images: None,
                    order: (idx + 1) as i32,
                })
                .collect();
            Some(Json(opts)) // 假设 options 类型是 Option<Json<Vec<QuestionOption>>>
        } else {
            None
        }
    } else {
        None
    };

    CreateQuestionReq {
        question_cate_id: task_info.question_cate_id as i32,
        source_id: parent_id,
        question_type_id,
        question_tag_ids: None,
        author_id: Some(meta::TEMP_ADMIN_ID),
        title: raw.stem.clone(),
        content_plain: Some(question::to_plain_text(&raw.stem)),
        comment: None,
        difficulty_level: markdown_parse::get_difficulty_level(&raw.table),
        images: None,
        options,
        options_layout: Some(1),
        answer: Some(raw.answer),
        knowledge: Some(raw.knowledge),
        analysis: Some(Json(Content {
            content: raw.analysis,
            images: None,
        })),
        process: Some(Json(Content {
            content: raw.detail,
            images: None,
        })),
        remark: Some("题目上传添加".to_string()),
    }
}
