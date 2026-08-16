use crate::api::textbook::{CreateTextbookReq, TextbookResp};
use crate::app::config::AppState;
use crate::constant;
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;
use crate::model::textbook::Textbook;
use crate::util::error::AppError;
use actix_web::web;
use log::error;
use sqlx::PgPool;
use std::collections::HashMap;

// 根据深度和父级关系将列表组合为有层级关系的列表
pub fn get_levels_by_parent_id(
    map: &HashMap<i32, Vec<Textbook>>,
    current_parent_id: i32,
    safe_depth: u32,
) -> Vec<TextbookResp> {
    // 递归结束
    if safe_depth == 0 {
        return vec![];
    }

    let mut res: Vec<TextbookResp> = vec![];

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
                table_name: Some("textbook".to_string()),
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
        let parent_id = row.parent_id.unwrap_or(0);
        // 使用 entry API 更优雅地处理“不存在则创建，存在则修改”
        map.entry(parent_id).or_default().push(row);
    }
    map
}

// 根据深度获取菜单列表, 待数据稳定后该接口需要缓存, 暂时因为表比较小可以不关注
pub async fn list_all(
    app_state: web::Data<AppState>,
    depth: u32,
) -> Result<Vec<TextbookResp>, AppError> {
    // 限制获取数据的最大层级
    let safe_depth = depth.min(constant::textbook::MAX_DEPTH);

    let rows = Textbook::find_all_by_depth(&app_state.get_ref().db, safe_depth)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            AppError::db_error("导航查询失败")
        })?;

    // 1. 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

    // 2. 从根节点（parent_id=0 是根）递归构建
    Ok(get_levels_by_parent_id(&map, 0, safe_depth))
}

// 根据父级标识获取子菜单列表
pub async fn list_level(
    app_state: web::Data<AppState>,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, AppError> {
    let rows = Textbook::find_list_by_parent_id(&app_state.get_ref().db, parent_id as i32)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            AppError::db_error("导航菜单查询失败")
        })?;

    Ok(rows.into_iter().map(|row| to_resp(row)).collect())
}

// 根据父标识列出所有题型列表
pub async fn list_children(
    app_state: web::Data<AppState>,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, AppError> {
    let db = &app_state.get_ref().db;

    // 获取原始列表
    let children_rows = Textbook::find_all_by_parent_id(db, parent_id as i32)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
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
    let ck_rows = ChapterKnowledge::find_by_ids(db, relation_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {:?}", e);
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
    let q_rows = QuestionCate::find_all_by_related_ids(db, bridge_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {:?}", e);
            AppError::db_error("题型查询失败")
        })?;

    let mut question_id_map: HashMap<i32, Vec<QuestionCate>> = HashMap::new();
    for row in q_rows {
        question_id_map.entry(row.related_id).or_default().push(row);
    }

    // 回填数据
    fill_question_cate(&relation_map, &question_id_map, &mut resp);

    Ok(resp)
}

fn fill_question_cate(
    relation_map: &HashMap<i32, Vec<i32>>,
    question_id_map: &HashMap<i32, Vec<QuestionCate>>,
    resp: &mut Vec<TextbookResp>,
) {
    for item in resp.iter_mut() {
        if item.path_depth != Some(7) && item.children.is_some() {
            fill_question_cate(
                relation_map,
                question_id_map,
                item.children.as_mut().unwrap(),
            );
            continue;
        }

        // 获取对应的关联 ID 列表引用
        if let Some(rel_ids) = relation_map.get(&item.id) {
            let row_children = item.children.get_or_insert_with(Vec::new);
            // 第8层菜单是拼接的题型列表
            for &rel_id in rel_ids {
                if let Some(questions) = question_id_map.get(&rel_id) {
                    // 直接遍历 questions 并克隆数据
                    for q in questions {
                        row_children.push(TextbookResp {
                            id: q.id,
                            path_type: constant::textbook::PATH_TYPE_COMMON.to_string(),
                            parent_id: None,
                            label: q.label.clone(),
                            key: format!("{}-{}", item.key, q.id), // 题型本身没有key， 拼接一个
                            sort_order: q.sort_order,
                            path_depth: None,
                            path: "".to_string(), // 题型不需要路径
                            table_name: Some("question_cate".to_string()),
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
            error!("Error searching textbook: {:?}", e);
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
pub async fn add(app_state: web::Data<AppState>, req: CreateTextbookReq) -> Result<i32, AppError> {
    let db = &app_state.get_ref().db;

    if req.id.is_some() {
        check_parent_and_label_is_exists(db, req.parent_id, req.label.as_str(), None).await?;
    }

    let row_id = Textbook::save(db, req).await.map_err(|e| {
        error!("Error inserting textbook: {:?}", e);
        AppError::db_error("菜单添加失败")
    })?;

    Ok(row_id)
}

// 数据库结构映射返回, 不直接返回数据库结构对象
fn to_resp(row: Textbook) -> TextbookResp {
    TextbookResp {
        id: row.id,
        path_type: row.path_type,
        parent_id: row.parent_id,
        label: row.label,
        key: row.key,
        sort_order: row.sort_order,
        path_depth: row.path_depth,
        path: row.path,
        table_name: Some("textbook".to_string()),
        children: None,
    }
}

// 详情
pub async fn info(app_state: web::Data<AppState>, id: i32) -> Result<TextbookResp, AppError> {
    let row = Textbook::find_by_id(&app_state.get_ref().db, id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            AppError::not_found("数据不存在")
        })?;

    Ok(to_resp(row))
}

// 删除菜单-没有子菜单的菜单可以被删除
pub async fn delete(app_state: web::Data<AppState>, id: i32) -> Result<bool, AppError> {
    let info = info(app_state.clone(), id).await?;

    let db = &app_state.get_ref().db;

    // 菜单层级检查是否存在子菜单
    let row = Textbook::find_one_by_parent_id(db, info.id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
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
        let chapters = ChapterKnowledge::find_by_chapter_or_knowledge_id(db, info.id)
            .await
            .map_err(|e| {
                error!("Error searching textbook: {:?}", e);
                AppError::db_error("章节考点查询失败")
            })?;
        if !chapters.is_empty() {
            return Err(AppError::business_error(
                "章节小节和知识点还存在绑定关系, 不允许删除",
            ));
        }
    }

    let row = Textbook::delete(db, id).await.map_err(|e| {
        error!("Error deleting textbook: {:?}", e);
        AppError::db_error("菜单删除失败")
    })?;

    Ok(row > 0)
}
