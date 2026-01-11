use crate::api::other_dict::CreateTextbookDictReq;
use sqlx::{FromRow, PgPool};

/// 教材其它字典

#[derive(FromRow)]
pub struct TextbookDict {
    pub id: i32,
    pub textbook_id: i32,
    pub type_code: String,
    pub item_value: String,
    pub sort_order: i32,
}

impl TextbookDict {
    // 添加字典项
    pub async fn insert(pool: &PgPool, req: CreateTextbookDictReq) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO textbook_dict (textbook_id, type_code, item_value, sort_order)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(req.textbook_id)
        .bind(req.type_code)
        .bind(req.item_value)
        .bind(req.sort_order)
        .fetch_one(pool)
        .await
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
            SELECT id, textbook_id, type_code, item_value, sort_order 
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
