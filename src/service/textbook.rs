use crate::api::textbook::{CreateTextbookReq, TextbookResp, UpdateTextbookReq};
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::question_cate::QuestionCate;
use crate::model::textbook::Textbook;
use crate::{AppConfig, constant};
use actix_web::web;
use log::error;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

// 根据深度和父级关系将列表组合为有层级关系的列表
fn get_levels_by_parent_id(
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
fn to_level_map(rows: Vec<Textbook>) -> HashMap<i32, Vec<Textbook>> {
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
    app_conf: web::Data<AppConfig>,
    depth: u32,
) -> Result<Vec<TextbookResp>, Error> {
    // 限制获取数据的最大层级
    let safe_depth = depth.min(constant::textbook::MAX_DEPTH);

    let rows = Textbook::find_all_by_depth(&app_conf.get_ref().db, safe_depth)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    // 1. 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

    // 2. 从根节点（parent_id=0 是根）递归构建
    Ok(get_levels_by_parent_id(&map, 0, safe_depth))
}

// 根据父级标识获取子菜单列表
pub async fn list_part(
    app_conf: web::Data<AppConfig>,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, Error> {
    let rows = Textbook::find_all_by_parent_id(&app_conf.get_ref().db, parent_id as i32)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    // 1. 建立父子索引映射
    let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

    // 2. 从根节点（parent_id=0 是根）递归构建
    Ok(get_levels_by_parent_id(&map, parent_id as i32, 2))
}

// 根据父标识列出所有题型列表
pub async fn list_children(
    app_conf: web::Data<AppConfig>,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, Error> {
    // 获取原始列表，注意加 mut
    let mut resp = list_part(app_conf.clone(), parent_id).await?;

    // 1. 提取关联 ID (利用迭代器链)
    let relation_ids: Vec<i32> = resp
        .iter()
        .filter(|item| item.path_depth == Some(6))
        .filter_map(|item| item.children.as_ref())
        .flat_map(|children| children.iter().map(|row| row.id))
        .collect();

    if relation_ids.is_empty() {
        return Ok(resp);
    }

    let db = &app_conf.get_ref().db;

    // 2. 查询中间关系表
    let rows = ChapterKnowledge::find_by_ids(db, relation_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let mut relation_map: HashMap<i32, i32> = HashMap::new();
    let mut bridge_ids = Vec::with_capacity(rows.len());
    for row in rows {
        bridge_ids.push(row.id);
        // 建立 原始ID -> 中间关联ID 的映射
        relation_map.insert(row.chapter_id, row.id);
        relation_map.insert(row.knowledge_id, row.id);
    }

    // 3. 查询题型分类
    let q_rows = QuestionCate::find_all_by_related_ids(db, bridge_ids)
        .await
        .map_err(|e| {
            error!("DB Error: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let mut question_id_map: HashMap<i32, Vec<QuestionCate>> = HashMap::new();
    for row in q_rows {
        question_id_map.entry(row.related_id).or_default().push(row);
    }

    // 4. 回填数据
    for item in resp.iter_mut() {
        // 使用 1.80+ 稳定的 let_chains
        if let Some(6) = item.path_depth
            && let Some(children_list) = &mut item.children
        {
            for row in children_list.iter_mut() {
                // 获取该行对应的关联 ID
                if let Some(&rel_id) = relation_map.get(&row.id) {
                    if let Some(questions) = question_id_map.get(&rel_id) {
                        // get_or_insert_with: 如果是 None 则初始化为 Vec，并返回可变引用
                        let row_children = row.children.get_or_insert_with(Vec::new);

                        for q in questions {
                            row_children.push(TextbookResp {
                                id: q.id,
                                path_type: constant::textbook::PATH_TYPE_COMMON.to_string(),
                                parent_id: None,
                                label: q.label.clone(),
                                key: String::new(),
                                sort_order: q.sort_order,
                                path_depth: None,
                                children: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(resp)
}

// 检查父级标识和名称是否存在, 不允许重复
async fn check_parent_and_label_is_exists(
    pool: &PgPool,
    parent_id: Option<i32>,
    label: &str,
    id: Option<i32>,
) -> Result<(), Error> {
    let row = Textbook::find_by_parent_and_label(pool, parent_id, label, id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    if row.is_none() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!("当前层级名称已存在: {}", label),
        ))
    }
}

// 添加
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateTextbookReq,
) -> Result<TextbookResp, Error> {
    let db = &app_conf.get_ref().db;

    check_parent_and_label_is_exists(db, req.parent_id, req.label.as_str(), None).await?;

    let row = Textbook::insert(db, req).await.map_err(|e| {
        error!("Error inserting textbook: {:?}", e);
        Error::new(ErrorKind::Other, "添加失败")
    })?;

    Ok(to_resp(row))
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
        children: None,
    }
}

// 详情
pub async fn info(app_conf: web::Data<AppConfig>, id: i32) -> Result<TextbookResp, Error> {
    let row = Textbook::find_by_id(&app_conf.get_ref().db, id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "数据不存在")
        })?;

    Ok(to_resp(row))
}

pub async fn info_list_by_ids(
    app_conf: web::Data<AppConfig>,
    ids: Vec<i32>,
) -> Result<Vec<TextbookResp>, Error> {
    let items = Textbook::find_by_ids(&app_conf.get_ref().db, ids)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;

    let res: Vec<TextbookResp> = items.into_iter().map(to_resp).collect();

    Ok(res)
}

// 编辑
// 编辑如果调整了父级id则所有的子级深度都需要更新, 更新的基准是按父级深度依次加1
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: UpdateTextbookReq,
) -> Result<TextbookResp, Error> {
    // 不允许自己挂载自己
    let req_parent_id = req.parent_id.unwrap_or(0);
    if req_parent_id == req.id {
        return Err(Error::new(ErrorKind::Other, "父级不能是自己"));
    }

    let old_row = info(app_conf.clone(), req.id).await?;

    let db = &app_conf.get_ref().db;

    check_parent_and_label_is_exists(db, req.parent_id, req.label.as_str(), Some(req.id)).await?;

    let is_parent_changed = req.parent_id != old_row.parent_id;

    // 深度, 编辑时默认为表里的深度, 如果父级变化则使用最新的相对父级深度
    let path_depth = if req_parent_id > 0 {
        let new_parent_row = info(app_conf.clone(), req_parent_id).await?;
        new_parent_row.path_depth.unwrap_or(0) + 1
    } else {
        1
    };

    // path_type
    let path_type = if req.path_type.is_none() {
        old_row.path_type
    } else {
        req.path_type.unwrap()
    };

    // 当父id变更时, 检查是否构成环
    if is_parent_changed && req_parent_id > 0 {
        let exist = Textbook::is_descendant(db, req.id, req_parent_id)
            .await
            .map_err(|e| {
                error!("Error searching textbook: {:?}", e);
                Error::new(ErrorKind::Other, "查询失败")
            })?;
        if exist {
            return Err(Error::new(
                ErrorKind::Other,
                "当前父级跟所选子级列表存在交叉, 不支持挂载",
            ));
        }
    }

    // 这部分更新使用事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Error beginning transaction: {}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    let row = Textbook::update(
        &mut *tx,
        req.id,
        req.parent_id,
        req.label.as_str(),
        req.sort_order,
        path_depth,
        path_type.as_str(),
    )
    .await
    .map_err(|e| {
        error!("Error updating textbook: {:?}", e);
        Error::new(ErrorKind::Other, "编辑失败")
    })?;

    // 所有子孙节点深度同步更新
    if is_parent_changed {
        let _ = Textbook::update_descendant_depth(
            &mut *tx,
            req.id,
            req.parent_id,
            path_depth,
            path_type.as_str(),
        )
        .await
        .map_err(|e| {
            error!("Error updating descendant depth: {:?}", e);
            Error::new(ErrorKind::Other, "更新失败")
        })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Error committing transaction: {}", e);
        Error::new(ErrorKind::Other, "更新失败")
    })?;

    Ok(to_resp(row))
}

// 删除菜单-没有子菜单的菜单可以被删除
pub async fn delete(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    let info = info(app_conf.clone(), id).await?;

    let db = &app_conf.get_ref().db;

    // 菜单层级检查是否存在子菜单
    let row = Textbook::find_by_parent_id(db, info.id)
        .await
        .map_err(|e| {
            error!("Error searching textbook: {:?}", e);
            Error::new(ErrorKind::Other, "查询失败")
        })?;
    if row.is_some() {
        return Err(Error::new(ErrorKind::Other, "该层级存在子菜单, 不允许删除"));
    }

    // 检查第7级菜单是否有子菜单
    if let Some(path_depth) = info.path_depth
        && path_depth == 7
    {
        // 检查该菜单是否关联过
        let chapter = ChapterKnowledge::find_by_chapter_or_knowledge_id(db, info.id)
            .await
            .map_err(|e| {
                error!("Error searching textbook: {:?}", e);
                Error::new(ErrorKind::Other, "查询失败")
            })?;
        if chapter.is_some() {
            return Err(Error::new(
                ErrorKind::Other,
                "章节小节和知识点还存在绑定关系, 不允许删除",
            ));
        }
    }

    let row = Textbook::delete(db, id).await.map_err(|e| {
        error!("Error deleting textbook: {:?}", e);
        Error::new(ErrorKind::Other, "删除失败")
    })?;

    Ok(row > 0)
}
