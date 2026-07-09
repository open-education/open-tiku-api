use crate::api::textbook::{CreateTextbookReq, TextbookResp};
use crate::model::textbook::Textbook;
use crate::service::textbook::{get_levels_by_parent_id, to_level_map};
use crate::{AppConfig, constant};
use log::error;
use sqlx::PgPool;
use std::collections::HashMap;

// 补充导航 path 路径值
pub async fn path(conf: &AppConfig) {
    // 限制获取数据的最大层级
    let safe_depth = constant::textbook::MAX_DEPTH;
    let rows = if let Ok(rows) = Textbook::find_all_by_depth(&conf.db, safe_depth).await {
        rows
    } else {
        error!("Error searching textbook");
        return;
    };

    // 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

    // 从根节点（parent_id=0 是根）递归构建
    let mut resp: Vec<TextbookResp> = get_levels_by_parent_id(&map, 0, safe_depth);
    for node in resp.iter_mut() {
        fix_node_path(node, String::new());
    }

    // 保存所有节点到数据库
    for node in resp.iter() {
        save_node_recursive(&conf.db, node).await;
    }
}

/// 递归为节点及其子节点补齐 path 字段
fn fix_node_path(node: &mut TextbookResp, parent_path: String) {
    // 设置当前节点的 path 为父路径（不包含当前节点）
    node.path = parent_path.clone();

    // 如果有子节点，递归处理
    if let Some(children) = &mut node.children {
        for child in children.iter_mut() {
            // 子节点的父路径 = 当前父路径 + "/" + 当前节点id
            let child_parent_path = if parent_path.is_empty() {
                // 如果父路径为空，直接使用 "/id"
                format!("/{}", node.id)
            } else {
                // 如果父路径不为空，追加 "/id"
                format!("{}/{}", parent_path, node.id)
            };
            fix_node_path(child, child_parent_path);
        }
    }
}

/// 递归保存节点及其所有子节点到数据库
async fn save_node_recursive(pool: &PgPool, node: &TextbookResp) {
    // 保存当前节点
    let req = CreateTextbookReq {
        id: Some(node.id),
        parent_id: node.parent_id,
        label: node.label.clone(),
        path_depth: node.path_depth,
        sort_order: node.sort_order,
        path_type: Some(node.path_type.clone()),
        path: node.path.clone(),
    };

    if let Err(e) = Textbook::save(pool, req).await {
        error!("Error saving textbook id={}: {:?}", node.id, e);
        return;
    }

    // 如果有子节点，使用循环保存，避免异步递归
    if let Some(children) = &node.children {
        for child in children {
            Box::pin(save_node_recursive(pool, child)).await;
        }
    }
}
