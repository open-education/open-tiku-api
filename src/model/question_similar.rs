use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, QueryBuilder, Transaction};

/// 变式题
#[allow(dead_code)]
#[derive(FromRow)]
pub struct QuestionSimilar {
    pub id: i64,
    pub question_type: i16,
    pub question_id: i64,
    pub child_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuestionSimilarType {
    Similar = 1,                  // 变式题
    OriginalTextbookQuestion = 2, // 课本原题
}

impl QuestionSimilarType {
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}

impl QuestionSimilar {
    /// 建立题目关联
    pub async fn insert(
        pool: &PgPool,
        question_id: i64,
        child_id: i64,
        question_type: i16,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO question_similar (question_id, child_id, question_type)
            VALUES ($1, $2, $3)
            ON CONFLICT (question_id, child_id) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(question_id)
        .bind(child_id)
        .bind(question_type)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            row.get::<i64, _>("id")
        })
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    /// 批量建立题目关联
    /// 关联的数量是上一步添加的变式题数量决定的, 而且只有id, 考虑到每次上传的题目我们会控制不要太大, 一次性写入变式题应该暂时不会有什么问题
    pub async fn batch_insert(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        pairs: Vec<(i64, i64, i16)>,
    ) -> Result<(), sqlx::Error> {
        // 空参数处理外面嵌套少一些
        if pairs.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO question_similar (question_id, child_id, question_type) ",
        );

        query_builder.push_values(pairs, |mut b, (question_id, child_id, question_type)| {
            b.push_bind(question_id)
                .push_bind(child_id)
                .push_bind(question_type);
        });

        query_builder.push(" ON CONFLICT (question_id, child_id) DO NOTHING");

        query_builder.build().execute(&mut **tx).await?;
        Ok(())
    }
}
