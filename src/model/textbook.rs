use crate::api::textbook::CreateTextbookReq;
use sqlx::{Executor, FromRow, PgPool, Postgres};

// 教材信息
// 如果要支持事务和非事务的方式查询, 可以参考这个写法, 实际上大部分是不需要关注事务的

#[derive(FromRow)]
pub struct Textbook {
    // SERIAL 对应 Rust 的 i32 (Postgres INTEGER)
    pub id: i32,

    // 路径类型
    pub path_type: String,

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
            INSERT INTO textbook (parent_id, label, key, path_depth, sort_order, path_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(data.parent_id)
        .bind(&data.label)
        .bind(Self::get_label_key(data.parent_id, &data.label))
        .bind(data.path_depth)
        .bind(data.sort_order)
        .bind(data.path_type)
        .fetch_one(pool)
        .await
    }

    /// 修改记录
    pub async fn update<'e, E>(
        executor: E,
        id: i32,
        parent_id: Option<i32>,
        label: &str,
        sort_order: i32,
        path_depth: i32,
        path_type: &str,
    ) -> Result<Self, sqlx::Error>
    where
        // 使用此约束可以同时接收 &PgPool 和 &mut Transaction
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query_as::<_, Self>(
            r#"
        UPDATE textbook
        SET parent_id = $2, label = $3, sort_order = $4, path_depth = $5, path_type = $6
        WHERE id = $1
        RETURNING *
        "#,
        )
        .bind(id)
        .bind(parent_id)
        .bind(label)
        .bind(sort_order)
        .bind(path_depth)
        .bind(path_type)
        .fetch_one(executor)
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
            SELECT id, parent_id, label, key, path_depth, sort_order, path_type
            FROM textbook
            WHERE parent_id = $1
            
            UNION ALL
            
            -- 递归部分：关联子节点
            SELECT t.id, t.parent_id, t.label, t.key, t.path_depth, t.sort_order, t.path_type
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

    /// 新的父节点是否是后代
    /// 检查 potential_parent_id 是否是当前 target_id 的子孙节点
    /// 如果返回 true，说明会形成环，禁止更新
    pub async fn is_descendant(
        pool: &PgPool,
        target_id: i32,
        potential_parent_id: i32,
    ) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE sub AS (
                -- 从当前节点的子级开始找
                SELECT id FROM textbook WHERE parent_id = $1
                UNION ALL
                SELECT t.id FROM textbook t JOIN sub s ON t.parent_id = s.id
            )
            SELECT EXISTS (SELECT 1 FROM sub WHERE id = $2)
            "#,
            target_id,           // $1: 当前节点 ID
            potential_parent_id  // $2: 想要设置的新父级 ID
        )
        .fetch_one(pool)
        .await?;

        Ok(exists.unwrap_or(false))
    }

    /// 根据已知的新深度更新节点及其所有后代，返回影响行数
    pub async fn update_descendant_depth<'e, E>(
        executor: E,
        target_id: i32,
        new_parent_id: Option<i32>,
        new_depth: i32, // 你已经知道的当前节点新深度
        new_path_type: &str,
    ) -> Result<u64, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let result = sqlx::query!(
            r#"
            WITH RECURSIVE tree AS (
                -- 1. 起点：当前节点，相对层级差设为 0
                SELECT id, 0 AS offset_level
                FROM textbook
                WHERE id = $1
                UNION ALL
                -- 2. 递归：所有后代，层级差逐级递增
                SELECT t.id, s.offset_level + 1
                FROM textbook t
                JOIN tree s ON t.parent_id = s.id
            )
            -- 3. 批量更新
            UPDATE textbook AS t
            SET 
                parent_id = CASE WHEN t.id = $1 THEN $2 ELSE t.parent_id END,
                -- 深度 = 给定的新深度 + 相对当前节点的偏移量
                path_depth = $3 + tree.offset_level,
                path_type = $4
            FROM tree
            WHERE t.id = tree.id
            "#,
            target_id,     // $1
            new_parent_id, // $2
            new_depth,     // $3
            new_path_type, // $4
        )
        .execute(executor)
        .await?;

        Ok(result.rows_affected())
    }
}
