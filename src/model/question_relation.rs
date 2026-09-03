use crate::enums::question::QuestionRelationType;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, QueryBuilder, Transaction};

// 题目关联关系

#[allow(dead_code)]
#[derive(FromRow)]
pub struct QuestionRelation {
    pub id: i64,
    pub question_type: i16,
    pub question_id: i64,
    pub child_id: i64,
    pub created_at: DateTime<Utc>,
}

impl QuestionRelation {
    pub async fn insert(
        pool: &PgPool,
        question_id: i64,
        child_id: i64,
        question_type: i16,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            r#"
            INSERT INTO question_relation (question_id, child_id, question_type)
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

        Ok(id)
    }

    // 批量建立题目关联
    // 关联的数量是上一步添加的变式题数量决定的, 而且只有id, 考虑到每次上传的题目我们会控制不要太大, 一次性写入变式题应该暂时不会有什么问题
    pub async fn batch_insert(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        pairs: Vec<(i64, i64, i16)>,
    ) -> Result<(), sqlx::Error> {
        // 空参数处理外面嵌套少一些
        if pairs.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO question_relation (question_id, child_id, question_type) ",
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

    // 通过母题标识查找变式题标识
    pub async fn find_original_by_base_id(pool: &PgPool, id: i64) -> Result<Vec<i64>, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT child_id
            FROM question_relation
            WHERE question_id = $1 AND question_type = $2
            "#,
        )
        .bind(id)
        .bind(QuestionRelationType::Original.as_i16())
        .fetch_all(pool)
        .await
    }

    // 通过变式题标识查找母题
    pub async fn find_base_by_similar_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT question_id
            FROM question_relation
            WHERE child_id = $1 AND question_type = $2
            "#,
        )
        .bind(id)
        .bind(QuestionRelationType::Similar.as_i16())
        .fetch_optional(pool)
        .await
    }
}
