use crate::api::textbook::{CreateTextbookReq, TextbookResp, UpdateTextbookReq};
use crate::model::chapter_knowledge::ChapterKnowledge;
use crate::model::textbook::Textbook;
use crate::{constant, AppConfig};
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

    match Textbook::find_all_by_depth(&app_conf.get_ref().db, safe_depth).await {
        Ok(rows) => {
            // 1. 建立父子索引映射
            let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

            // 2. 从根节点（parent_id=0 是根）递归构建
            Ok(get_levels_by_parent_id(&map, 0, safe_depth))
        }
        Err(e) => {
            error!("Database list query error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "查询失败"))
        }
    }
}

// 根据父级标识获取子菜单列表
pub async fn list_part(
    app_conf: web::Data<AppConfig>,
    parent_id: u32,
) -> Result<Vec<TextbookResp>, Error> {
    match Textbook::find_all_by_parent_id(&app_conf.get_ref().db, parent_id as i32).await {
        Ok(rows) => {
            // 1. 建立父子索引映射
            let map: HashMap<i32, Vec<Textbook>> = to_level_map(rows);

            // 2. 从根节点（parent_id=0 是根）递归构建
            Ok(get_levels_by_parent_id(&map, parent_id as i32, 2))
        }
        Err(e) => {
            error!("Database list part query error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "查询失败"))
        }
    }
}

// 检查父级标识和名称是否存在, 不允许重复
async fn check_parent_and_label_is_exists(
    pool: &PgPool,
    parent_id: Option<i32>,
    label: &str,
    id: Option<i32>,
) -> Result<(), Error> {
    match Textbook::find_by_parent_and_label(&pool, parent_id, label, id).await {
        Ok(row) => {
            if let Some(_) = row {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("当前层级名称已存在: {}", label),
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            error!("Database check parent and label is exists error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "查询失败"))
        }
    }
}

// 添加
pub async fn add(
    app_conf: web::Data<AppConfig>,
    req: CreateTextbookReq,
) -> Result<TextbookResp, Error> {
    check_parent_and_label_is_exists(
        &app_conf.get_ref().db,
        req.parent_id,
        req.label.as_str(),
        None,
    )
    .await?;

    match Textbook::insert(&app_conf.get_ref().db, req).await {
        Ok(row) => Ok(to_resp(row)),
        Err(e) => {
            error!("Database add save error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "添加失败"))
        }
    }
}

// 数据库结构映射返回, 不直接返回数据库结构对象
fn to_resp(row: Textbook) -> TextbookResp {
    TextbookResp {
        id: row.id,
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
    match Textbook::find_by_id(&app_conf.get_ref().db, id).await {
        Ok(row) => Ok(to_resp(row)),
        Err(e) => {
            error!("Database edit query error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "数据不存在"))
        }
    }
}

pub async fn info_list_by_ids(
    app_conf: web::Data<AppConfig>,
    ids: Vec<i32>,
) -> Result<Vec<TextbookResp>, Error> {
    match Textbook::find_by_ids(&app_conf.get_ref().db, ids).await {
        Ok(items) => {
            let mut res: Vec<TextbookResp> = vec![];
            for item in items {
                res.push(to_resp(item));
            }
            Ok(res)
        }
        Err(err) => {
            error!("get all by ids err: {}", err);
            Err(Error::new(ErrorKind::Other, ""))
        }
    }
}

// 编辑
pub async fn edit(
    app_conf: web::Data<AppConfig>,
    req: UpdateTextbookReq,
) -> Result<TextbookResp, Error> {
    let _ = info(app_conf.clone(), req.id).await?;

    check_parent_and_label_is_exists(
        &app_conf.get_ref().db,
        req.parent_id,
        req.label.as_str(),
        Some(req.id),
    )
    .await?;

    match Textbook::update(&app_conf.get_ref().db, req).await {
        Ok(row) => Ok(to_resp(row)),
        Err(e) => {
            error!("Database edit update error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "编辑失败"))
        }
    }
}

// 删除菜单-没有子菜单的菜单可以被删除
pub async fn delete(app_conf: web::Data<AppConfig>, id: i32) -> Result<bool, Error> {
    let info = info(app_conf.clone(), id).await?;

    // 菜单层级检查是否存在子菜单
    match Textbook::find_by_parent_id(&app_conf.get_ref().db, info.id).await {
        Ok(row) => {
            if let Some(_) = row {
                return Err(Error::new(ErrorKind::Other, "该层级存在子菜单, 不允许删除"));
            }
        }
        Err(e) => {
            error!("Database find parent_id query error: {:?}", e);
            return Err(Error::new(ErrorKind::Other, "删除失败"));
        }
    }

    // 检查第7级菜单是否有子菜单
    if let Some(path_depth) = info.path_depth
        && path_depth == 7
    //todo 暂时写死
    {
        // 检查该菜单是否关联过
        match ChapterKnowledge::find_by_chapter_or_knowledge_id(&app_conf.get_ref().db, info.id)
            .await
        {
            Ok(chapter) => {
                if let Some(_) = chapter {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "章节小节和知识点还存在绑定关系, 不允许删除",
                    ));
                }
            }
            Err(err) => {
                error!("textbook chapter knowledge id query error: {:?}", err);
                return Err(Error::new(ErrorKind::Other, "查询失败"));
            }
        }
    }

    match Textbook::delete(&app_conf.get_ref().db, id).await {
        Ok(row) => Ok(row > 0),
        Err(e) => {
            error!("Database delete error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "删除失败"))
        }
    }
}
