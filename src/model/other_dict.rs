use crate::api::other_dict::CreateTextbookDictReq;
use sqlx::{FromRow, PgPool};

/// 教材其它字典

#[derive(FromRow, Clone)]
pub struct TextbookDict {
    pub id: i32,
    pub textbook_id: i32,
    pub type_code: String,
    pub item_value: String,
    pub sort_order: i32,
    pub is_select: bool,
}

impl TextbookDict {
    // 添加字典项
    pub async fn insert(pool: &PgPool, req: CreateTextbookDictReq) -> Result<i32, sqlx::Error> {
        let id: i32 = sqlx::query_scalar(
            r#"
        INSERT INTO textbook_dict (id, textbook_id, type_code, item_value, sort_order, is_select)
        VALUES (
            COALESCE(NULLIF($1, 0), nextval('textbook_dict_id_seq')),
            $2, $3, $4, $5, $6
        )
        ON CONFLICT (id) DO UPDATE SET
            textbook_id = EXCLUDED.textbook_id,
            type_code = EXCLUDED.type_code,
            item_value = EXCLUDED.item_value,
            sort_order = EXCLUDED.sort_order,
            is_select = EXCLUDED.is_select
        RETURNING id
        "#,
        )
        .bind(req.id)
        .bind(req.textbook_id)
        .bind(req.type_code)
        .bind(req.item_value)
        .bind(req.sort_order)
        .bind(req.is_select)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    // 根据类型和字典值查询是否已存在
    pub async fn find_by_unique(
        pool: &PgPool,
        textbook_id: i32,
        type_code: &str,
        item_value: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM textbook_dict WHERE textbook_id = $1 AND type_code = $2 AND item_value = $3",
        ).bind(textbook_id)
        .bind(type_code)
        .bind(item_value)
        .fetch_optional(pool)
        .await
    }

    // 根据类型标识查询列表 (例如获取所有 'question_type')
    pub async fn find_by_textbook_and_type(
        pool: &PgPool,
        textbook_id: i32,
        type_code: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT id, textbook_id, type_code, item_value, sort_order, is_select 
            FROM textbook_dict 
            WHERE textbook_id = $1 AND type_code = $2
            ORDER BY sort_order
            "#,
        )
        .bind(textbook_id)
        .bind(type_code)
        .fetch_all(pool)
        .await
    }

    // 删除特定字典项
    pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM textbook_dict WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}
