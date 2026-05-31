use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, QueryBuilder, Transaction};

/// 变式题
#[allow(dead_code)]
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

    /// 批量建立题目关联
    /// 关联的数量是上一步添加的变式题数量决定的, 而且只有id, 考虑到每次上传的题目我们会控制不要太大, 一次性写入变式题应该暂时不会有什么问题
    pub async fn batch_insert(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        pairs: Vec<(i64, i64)>,
    ) -> Result<(), sqlx::Error> {
        // 空参数处理外面嵌套少一些
        if pairs.is_empty() {
            return Ok(());
        }

        let mut query_builder =
            QueryBuilder::new("INSERT INTO question_similar (question_id, child_id) ");

        query_builder.push_values(pairs, |mut b, (question_id, child_id)| {
            b.push_bind(question_id).push_bind(child_id);
        });

        query_builder.push(" ON CONFLICT (question_id, child_id) DO NOTHING");

        query_builder.build().execute(&mut **tx).await?;
        Ok(())
    }
}
