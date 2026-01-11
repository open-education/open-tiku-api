use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

/// 变式题
#[derive(FromRow)]
pub struct QuestionSimilar {
    pub id: i64, // BIGSERIAL 对应 i64
    pub question_id: i64,
    pub child_id: i64,
    pub created_at: DateTime<Utc>, // 对应 TIMESTAMPTZ
}

impl QuestionSimilar {
    /// 建立题目关联
    pub async fn insert(
        pool: &PgPool,
        question_id: i64,
        child_id: i64,
    ) -> Result<i64, sqlx::Error> {
        // 使用 ON CONFLICT DO NOTHING 防止重复关联报错
        // RETURNING id 返回生成的主键
        let row = sqlx::query!(
            r#"
            INSERT INTO question_similar (question_id, child_id)
            VALUES ($1, $2)
            ON CONFLICT (question_id, child_id) DO NOTHING
            RETURNING id
            "#,
            question_id,
            child_id
        )
        .fetch_optional(pool) // 使用 optional 因为 DO NOTHING 可能不返回行
        .await?;

        // 如果已存在则返回 0 或错误，取决于你的业务逻辑
        Ok(row.map(|r| r.id).unwrap_or(0))
    }
}
