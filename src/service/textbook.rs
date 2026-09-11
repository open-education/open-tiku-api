use crate::api::req::textbook::CreateTextbookReq;
use crate::api::resp::textbook::TextbookResp;
use crate::app::conf::AppState;
use crate::constant;
use crate::constant::cache::TEXTBOOK_CACHE_PREFIX;
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;
use crate::model::textbook::Textbook;
use crate::util::cache;
use crate::util::error::AppError;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tracing::error;

// 根据深度和父级关系将列表组合为有层级关系的列表
pub fn get_levels_by_parent_id(
    map: &HashMap<i32, Vec<Textbook>>,
    current_parent_id: i32,
    safe_depth: u32,
) -> Vec<TextbookResp> {
    // 递归结束
    if safe_depth == 0 {
        return Vec::new();
    }

    let mut res: Vec<TextbookResp> = Vec::new();

    // 查找以 current_parent_id 为父节点的所有子项
    if let Some(items) = map.get(&current_parent_id) {
        for item in items {
            let mut info = TextbookResp {
                id: item.id,
                path_type: item.path_type.clone(),
                parent_id: item.parent_id,
                label: item.label.clone(),
                key: item.key.clone(),
                sort_order: item.sort_order,
                path_depth: item.path_depth,
                path: item.path.clone(),
                table_name: Some(String::from("textbook")),
                children: None,
            };

            // 关键: 递归查找当前项(subject.id)的子节点
            let children = get_levels_by_parent_id(map, item.id, safe_depth - 1);
            if !children.is_empty() {
                info.children = Some(children);
            }

            res.push(info);
        }
    }
    res
}

// 将教材字典类表变更为字典类型
pub fn to_level_map(rows: Vec<Textbook>) -> HashMap<i32, Vec<Textbook>> {
    let mut map: HashMap<i32, Vec<Textbook>> = HashMap::with_capacity(rows.len());
    for row in rows {
        let parent_id = row.parent_id.unwrap_or_default();
        // 使用 entry API 更优雅地处理“不存在则创建，存在则修改”
        map.entry(parent_id).or_default().push(row);
    }
    map
}

// 根据深度获取菜单列表, 待数据稳定后该接口需要缓存, 暂时因为表比较小可以不关注
pub async fn list_all(app_state: &AppState, depth: u32) -> Result<Vec<TextbookResp>, AppError> {
    // 限制获取数据的最大层级
    let safe_depth = depth.min(constant::textbook::MAX_DEPTH);

    let cache_key = format!("{TEXTBOOK_CACHE_PREFIX}:all:{depth}");
    match cache::get::<Vec<TextbookResp>>(&app_state.sqlite, &cache_key).await {
        Ok(resp) => return Ok(resp),
        Err(err) => {
            error!(
                "Get textbook list all cache key: {cache_key}, msg: {}",
                err.msg
            );
        }
    }

    let rows = Textbook::find_all_by_depth(&app_state.db, safe_depth)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {e}");
            AppError::db_error("导航查询失败")
        })?;

    // 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

    // 从根节点（parent_id=0 是根）递归构建
    let resp = get_levels_by_parent_id(&map, 0, safe_depth);

    cache::set::<Vec<TextbookResp>>(
        &app_state.sqlite,
        &cache_key,
        &resp,
        Duration::from_hours(24),
    )
    .await;

    Ok(resp)
}

// 根据父级标识获取子菜单列表
pub async fn list_level(
    app_state: &AppState,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, AppError> {
    let rows = Textbook::find_list_by_parent_id(&app_state.db, parent_id as i32)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {e}");
            AppError::db_error("导航菜单查询失败")
        })?;

    Ok(rows.into_iter().map(Into::into).collect())
}

// 根据父标识列出所有题型列表
pub async fn list_children(
    app_state: &AppState,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, AppError> {
    let cache_key = format!("{}:children:{}", TEXTBOOK_CACHE_PREFIX, parent_id);
    match cache::get::<Vec<TextbookResp>>(&app_state.sqlite, &cache_key).await {
        Ok(resp) => return Ok(resp),
        Err(err) => {
            error!(
                "Get textbook list children cache key: {cache_key}, msg: {}",
                err.msg
            );
        }
    }

    let db = &app_state.db;

    // 获取原始列表
    let children_rows = Textbook::find_all_by_parent_id(db, parent_id as i32)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {e}");
            AppError::db_error("菜单列表查询失败")
        })?;

    // 提取关联 ID (利用迭代器链)
    let relation_ids: Vec<i32> = children_rows
        .iter()
        .filter(|item| item.path_depth == Some(7))
        .map(|item| item.id)
        .collect();

    // 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(children_rows);

    let mut resp = get_levels_by_parent_id(&map, parent_id as i32, constant::textbook::MAX_DEPTH);

    if relation_ids.is_empty() {
        return Ok(resp);
    }

    // 查询中间关系表
    let ck_rows = ChapterKnowledge::find_by_ck_ids(db, &relation_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {e}");
            AppError::db_error("考点章节绑定关系查询失败")
        })?;

    // 目前的关联关系是 章节选题 -> 多个考点选题
    let mut relation_map: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut bridge_ids = Vec::with_capacity(ck_rows.len());
    for row in ck_rows {
        bridge_ids.push(row.id);
        // 建立 原始ID -> 中间关联ID 的映射
        // 使用 .entry().or_default() 自动处理 Vec 的初始化和推入
        relation_map.entry(row.chapter_id).or_default().push(row.id);
        relation_map
            .entry(row.knowledge_id)
            .or_default()
            .push(row.id);
    }

    // 查询题型分类
    let q_rows = QuestionCate::find_all_by_related_ids(db, &bridge_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {e}");
            AppError::db_error("题型查询失败")
        })?;

    let mut question_id_map: HashMap<i32, Vec<QuestionCate>> = HashMap::new();
    for row in q_rows {
        question_id_map.entry(row.related_id).or_default().push(row);
    }

    // 回填数据
    fill_question_cate(&relation_map, &question_id_map, &mut resp);

    cache::set::<Vec<TextbookResp>>(
        &app_state.sqlite,
        &cache_key,
        &resp,
        Duration::from_hours(24),
    )
    .await;

    Ok(resp)
}

fn fill_question_cate(
    relation_map: &HashMap<i32, Vec<i32>>,
    question_id_map: &HashMap<i32, Vec<QuestionCate>>,
    resp: &mut [TextbookResp],
) {
    // 分配 64 字节 整个递归或循环期间避免每次都向操作系统 Malloc
    let mut key_buf = String::with_capacity(64);
    let mut id_buf = itoa::Buffer::new(); // 引入栈 buffer 专门做极速数字解析

    for item in resp.iter_mut() {
        if let Some(ref mut children) = item.children {
            fill_question_cate(relation_map, question_id_map, children);
            continue;
        }

        if let Some(rel_ids) = relation_map.get(&item.id) {
            // 顺手计算出当前节点一共要 push 多少道题
            let total_questions: usize = rel_ids
                .iter()
                .filter_map(|id| question_id_map.get(id))
                .map(Vec::len)
                .sum();

            if total_questions == 0 {
                continue;
            }

            // 一步到位初始化或扩容到精准的最终大小
            let row_children = item
                .children
                .get_or_insert_with(|| Vec::with_capacity(total_questions));

            for &rel_id in rel_ids {
                if let Some(questions) = question_id_map.get(&rel_id) {
                    for q in questions {
                        // 极致重用 清理盘子 重置长度为0但容量不变 纯内存连续直写
                        key_buf.clear();
                        let id_str = id_buf.format(q.id); // 纯栈上的数字转字符

                        key_buf.push_str(&item.key);
                        key_buf.push('-');
                        key_buf.push_str(id_str);

                        row_children.push(TextbookResp {
                            id: q.id,
                            path_type: String::from(constant::textbook::PATH_TYPE_COMMON),
                            parent_id: None,
                            label: q.label.clone(),
                            key: key_buf.clone(),
                            sort_order: q.sort_order,
                            path_depth: None,
                            path: String::new(),
                            table_name: Some(String::from("question_cate")),
                            children: None,
                        });
                    }
                }
            }
        }
    }
}

// 检查父级标识和名称是否存在, 不允许重复
async fn check_parent_and_label_is_exists(
    pool: &PgPool,
    parent_id: Option<i32>,
    label: &str,
    id: Option<i32>,
) -> Result<(), AppError> {
    let row = Textbook::find_one_by_parent_and_label(pool, parent_id, label, id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {e}");
            AppError::db_error("菜单名称查询查询失败")
        })?;

    if row.is_none() {
        Ok(())
    } else {
        Err(AppError::business_error(
            format!("当前层级名称已存在: {}", label).as_str(),
        ))
    }
}

// 添加
pub async fn add(app_state: &AppState, req: CreateTextbookReq) -> Result<i32, AppError> {
    let db = &app_state.db;

    check_parent_and_label_is_exists(db, req.parent_id, req.label.as_str(), req.id).await?;

    let row_id = Textbook::save(db, req).await.map_err(|e| {
        error!("Error inserting textbook: {e}");
        AppError::db_error("菜单添加失败")
    })?;

    cache::delete_by_prefix(&app_state.sqlite, TEXTBOOK_CACHE_PREFIX).await;

    Ok(row_id)
}

// 详情
pub async fn info(app_state: &AppState, id: i32) -> Result<TextbookResp, AppError> {
    let row = Textbook::find_by_id(&app_state.db, id).await.map_err(|e| {
        error!("Error searching textbook: {e}");
        AppError::not_found("数据不存在")
    })?;

    Ok(row.into())
}

// 删除菜单-没有子菜单的菜单可以被删除
pub async fn delete(app_state: &AppState, id: i32) -> Result<bool, AppError> {
    let info = info(app_state, id).await?;

    let db = &app_state.db;

    // 菜单层级检查是否存在子菜单
    let row = Textbook::find_one_by_parent_id(db, info.id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {e}");
            AppError::db_error("菜单查询失败")
        })?;
    if row.is_some() {
        return Err(AppError::business_error("该层级存在子菜单, 不允许删除"));
    }

    // 检查第7级菜单是否有子菜单
    if let Some(path_depth) = info.path_depth
        && path_depth == 7
    {
        // 检查该菜单是否关联过
        let chapters = ChapterKnowledge::find_by_ck_id(db, info.id)
            .await
            .map_err(|e| {
                error!("Error searching textbook: {e}");
                AppError::db_error("章节考点查询失败")
            })?;
        if !chapters.is_empty() {
            return Err(AppError::business_error(
                "章节小节和知识点还存在绑定关系, 不允许删除",
            ));
        }
    }

    let row = Textbook::delete_by_id(db, id).await.map_err(|e| {
        error!("Error deleting textbook: {e}");
        AppError::db_error("菜单删除失败")
    })?;

    cache::delete_by_prefix(&app_state.sqlite, TEXTBOOK_CACHE_PREFIX).await;

    Ok(row > 0)
}
