use crate::api::req::question_cate::{CreateQuestionCateReq, QuestionCateListReq};
use crate::api::resp::question_cate::{QuestionCateListResp, QuestionCateResp};
use crate::api::resp::textbook::TextbookResp;
use crate::app::conf::AppState;
use crate::constant::cache::TEXTBOOK_CACHE_PREFIX;
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question::Question;
use crate::model::question_cate::QuestionCate;
use crate::service::textbook;
use crate::util::cache;
use crate::util::error::AppError;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::error;

// 题型列表
pub async fn list(
    app_state: &AppState,
    related_id: i32,
) -> Result<Vec<QuestionCateResp>, AppError> {
    let db = &app_state.db;

    let rows = QuestionCate::find_all_by_related_ids(db, &[related_id])
        .await
        .map_err(|err| {
            error!("error finding question cat: {err}");
            AppError::db_error("题型查询失败")
        })?;

    let res: Vec<QuestionCateResp> = rows.into_iter().map(Into::into).collect();

    Ok(res)
}

// 将完整的7层菜单构建成有第7层菜单的一维 Map 结构, 结构不再包含 子节点
fn build_seven_level_map(trees: Vec<TextbookResp>) -> HashMap<i32, TextbookResp> {
    let mut valid_ids = HashSet::new();
    let mut current_path = Vec::new();

    // 标记所有拥有第 7 层叶子节点的合法链路 ID
    fn collect_valid_ids(node: &TextbookResp, path: &mut Vec<i32>, valid_ids: &mut HashSet<i32>) {
        let depth = node.path_depth.unwrap_or(0);
        path.push(node.id);

        if depth == 7 {
            for id in path.iter() {
                valid_ids.insert(*id);
            }
        } else if let Some(children) = &node.children {
            for child in children {
                collect_valid_ids(child, path, valid_ids);
            }
        }
        path.pop();
    }

    for root in &trees {
        collect_valid_ids(root, &mut current_path, &mut valid_ids);
    }

    let mut global_flat_map = HashMap::new();

    // 打平并剥离 children 存入全局 Map 丢弃不完整的分支
    fn populate_flat_map(
        mut node: TextbookResp,
        valid_ids: &HashSet<i32>,
        flat_map: &mut HashMap<i32, TextbookResp>,
    ) {
        if !valid_ids.contains(&node.id) {
            return;
        }

        let node_id = node.id;

        // 使用 Option::take()
        // 这会把 children 的数据获取同时自动把原 node.children 原地设为 None
        let children = node.children.take();

        // 此时 node 内部的 children 已经是 None
        flat_map.insert(node_id, node);

        if let Some(children_vec) = children {
            for child in children_vec {
                populate_flat_map(child, valid_ids, flat_map);
            }
        }
    }

    for root in trees {
        populate_flat_map(root, &valid_ids, &mut global_flat_map);
    }

    global_flat_map
}

async fn get_seven_level_map(app_state: &AppState) -> Result<HashMap<i32, TextbookResp>, AppError> {
    // 获取完整的七级菜单信息, 内部已处理过缓存
    let depth: u32 = 7;
    let seven_resp: Vec<TextbookResp> = textbook::list_all(app_state, depth).await?;

    let cache_key = format!("{TEXTBOOK_CACHE_PREFIX}:seven:level:{depth}");
    match cache::get::<HashMap<i32, TextbookResp>>(&app_state.sqlite, &cache_key).await {
        Ok(resp) => return Ok(resp),
        Err(err) => {
            error!(
                "Get question cate seven list all cache key: {cache_key}, msg: {}",
                err.msg
            );
        }
    }

    // 然后建立第7层菜单跟它对应的 map
    let seven_map = build_seven_level_map(seven_resp);

    cache::set::<HashMap<i32, TextbookResp>>(
        &app_state.sqlite,
        &cache_key,
        &seven_map,
        Duration::from_hours(24),
    )
    .await;

    Ok(seven_map)
}

pub async fn list_all(
    app_state: &AppState,
    req: QuestionCateListReq,
) -> Result<QuestionCateListResp, AppError> {
    if req.ids.is_empty() {
        return Err(AppError::param_error("题型不能为空"));
    }

    let db = &app_state.db;

    // 题型分类信息本身不做缓存
    let rows = QuestionCate::find_by_ids(db, &req.ids)
        .await
        .map_err(|err| {
            error!("find question cate ids: {:?} row error: {err}", req.ids);
            AppError::db_error("查询题型分类出错")
        })?;
    if rows.is_empty() {
        return Err(AppError::not_found("题型分类不存在"));
    }
    // 得到关联 related_id 列表
    let mut related_ids: Vec<i32> = rows.iter().map(|item| item.related_id).collect();
    related_ids.sort_unstable();
    related_ids.dedup();
    let ck_rows = ChapterKnowledge::find_by_ids(db, &related_ids)
        .await
        .map_err(|err| {
            error!(
                "find question cate and chapter knowledge ids: {:?} row error: {err}",
                related_ids
            );
            AppError::db_error("获取题型菜单关联关系出错")
        })?;
    if ck_rows.is_empty() {
        return Err(AppError::not_found("题型菜单关联为空"));
    }
    let ck_map: HashMap<i32, &ChapterKnowledge> = ck_rows.iter().map(|ck| (ck.id, ck)).collect();

    // 获取扁平的7层菜单信息
    let seven_map = get_seven_level_map(app_state).await?;

    let mut info_map: HashMap<i32, QuestionCateResp> = HashMap::with_capacity(rows.len());
    let mut parent_map: HashMap<i32, TextbookResp> = HashMap::new();

    // 遍历每一行题型 双线展开并剪枝
    for row in rows {
        let info: QuestionCateResp = row.into();
        let info_id = info.id;
        let related_id = info.related_id;

        // 塞入第 8 层的题型字典
        info_map.insert(info_id, info);

        // 如果在关联表里找不到对应的关系 说明这行题型数据非法 直接跳过, 不影响前端展示题目本身
        let Some(ck_info) = ck_map.get(&related_id) else {
            continue;
        };

        for &start_pid in &[ck_info.chapter_id, ck_info.knowledge_id] {
            let mut current_parent_id = Some(start_pid);

            while let Some(pid) = current_parent_id {
                // 剪枝拦截 如果 parent_map 里有了 说明前人或另一条线的前期节点已经走过了
                if parent_map.contains_key(&pid) {
                    break;
                }

                if let Some(node) = seven_map.get(&pid) {
                    // 显式解引用并克隆，安全存入 parent_map
                    parent_map.insert(pid, (*node).clone());
                    current_parent_id = node.parent_id;
                } else {
                    break;
                }
            }
        }
    }

    let resp = QuestionCateListResp {
        info_map,
        parent_map,
    };

    Ok(resp)
}

// 添加题型
pub async fn add(app_state: &AppState, req: CreateQuestionCateReq) -> Result<i32, AppError> {
    let row_id = QuestionCate::save(&app_state.db, req)
        .await
        .map_err(|err| {
            error!("error adding question: {err}");
            AppError::db_error("题型添加失败")
        })?;

    cache::delete_by_prefix(&app_state.sqlite, TEXTBOOK_CACHE_PREFIX).await;

    Ok(row_id)
}

// 删除题型
pub async fn remove(app_state: &AppState, id: i32) -> Result<bool, AppError> {
    let db = &app_state.db;

    // 关联题目后就不允许删除了
    let exist = Question::exist_by_cate_id(db, id).await.map_err(|err| {
        error!("error finding exists question: {}", err);
        AppError::db_error("题型查询失败")
    })?;
    if exist {
        return Err(AppError::permission_denied("题型已关联题目, 不允许删除"));
    }

    let row = QuestionCate::delete(db, id).await.map_err(|err| {
        error!("error deleting question: {err}");
        AppError::db_error("题目删除失败")
    })?;

    cache::delete_by_prefix(&app_state.sqlite, TEXTBOOK_CACHE_PREFIX).await;

    Ok(row > 0)
}
