use crate::api::textbook::{CreateTextbookReq, UpdateTextbookReq};
use sqlx::{FromRow, PgPool};

// 教材信息

#[derive(FromRow)]
pub struct Textbook {
    // SERIAL 对应 Rust 的 i32 (Postgres INTEGER)
    pub id: i32,

    // REFERENCES 可能为空（根节点），所以使用 Option
    pub parent_id: Option<i32>,

    // VARCHAR 对应 String
    pub label: String,

    pub key: String,

    // INTEGER 对应 i32
    pub path_depth: Option<i32>,

    pub sort_order: i32,
}

// 类似 dao 逻辑直接实现即可
impl Textbook {
    // 每个菜单的标识, 可能暂时没什么需要
    fn get_label_key(parent_id: Option<i32>, label: &str) -> String {
        if let Some(parent_id) = parent_id {
            (&format!("{:x}", md5::compute(format!("{}_{}", parent_id, label)))[..10]).to_string()
        } else {
            (&format!("{:x}", md5::compute(label))[..10]).to_string()
        }
    }

    /// 新增记录
    /// 使用 RETURNING * 可以直接返回数据库生成后的完整对象（包含 id 和 created_at）
    pub async fn insert(pool: &PgPool, data: CreateTextbookReq) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO textbook (parent_id, label, key, path_depth, sort_order)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(data.parent_id)
        .bind(&data.label)
        .bind(Self::get_label_key(data.parent_id, &data.label))
        .bind(data.path_depth)
        .bind(data.sort_order)
        .fetch_one(pool)
        .await
    }

    /// 修改记录
    pub async fn update(pool: &PgPool, data: UpdateTextbookReq) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE textbook
            SET parent_id = $2, label = $3, sort_order = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(data.id)
        .bind(data.parent_id)
        .bind(data.label)
        .bind(data.sort_order)
        .fetch_one(pool)
        .await
    }

    /// 删除记录
    /// 返回 Result<()> 或受影响的行数
    pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM textbook WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// 场景：在指定目录下根据 id 查找
    pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM textbook WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    // 通过章节查询已关联的知识点类
    pub async fn find_by_ids(pool: &PgPool, ids: Vec<i32>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM textbook WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await
    }

    /// 场景：在指定目录下根据 parent_id 查找
    pub async fn find_by_parent_id(
        pool: &PgPool,
        parent_id: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM textbook WHERE parent_id = $1")
            .bind(parent_id)
            .fetch_optional(pool)
            .await
    }

    /// 场景：在指定目录下根据名称查找
    pub async fn find_by_parent_and_label(
        pool: &PgPool,
        parent_id: Option<i32>,
        label: &str,
        id: Option<i32>,
    ) -> Result<Option<Self>, sqlx::Error> {
        // 注意返回值改为 Option
        sqlx::query_as::<_, Self>(
            r#"
        SELECT * FROM textbook
        WHERE parent_id IS NOT DISTINCT FROM $1 
          AND label = $2
          AND ($3 IS NULL OR id <> $3)
        "#,
        )
        .bind(parent_id)
        .bind(label)
        .bind(id)
        .fetch_optional(pool) // 使用 fetch_optional
        .await
    }

    /// 根据深度限制获取教材层级关系列表 - all
    pub async fn find_all_by_depth(pool: &PgPool, depth: u32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM textbook WHERE path_depth <= $1 ORDER BY path_depth, sort_order",
        )
        .bind(depth as i32)
        .fetch_all(pool)
        .await
    }

    // 获取父级标识下面的层级, 没有控制层级
    pub async fn find_all_by_parent_id(
        pool: &PgPool,
        root_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        // 使用 WITH RECURSIVE 进行递归查询
        let rows = sqlx::query_as::<_, Self>(
            r#"
        WITH RECURSIVE tree AS (
            -- 锚点部分：选择起始节点（你想从哪个 parent_id 开始找）
            SELECT id, parent_id, label, key, path_depth, sort_order
            FROM textbook
            WHERE parent_id = $1
            
            UNION ALL
            
            -- 递归部分：关联子节点
            SELECT t.id, t.parent_id, t.label, t.key, t.path_depth, t.sort_order
            FROM textbook t
            INNER JOIN tree ON t.parent_id = tree.id
        )
        SELECT * FROM tree ORDER BY path_depth, sort_order;
        "#,
        )
        .bind(root_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
