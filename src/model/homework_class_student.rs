use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

#[derive(FromRow)]
pub struct HomeworkClassStudent {
    pub id: i64,
    pub homework_id: i64,
    pub student_id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl HomeworkClassStudent {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        records: &[Self],
    ) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb =
            QueryBuilder::new("INSERT INTO homework_class_student (homework_id, student_id) ");
        qb.push_values(records, |mut b, r| {
            b.push_bind(r.homework_id).push_bind(r.student_id);
        });
        Ok(qb.build().execute(&mut **tx).await?.rows_affected())
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Self>> {
        let row = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_class_student
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_by_homework_ids(pool: &PgPool, ids: Vec<i64>) -> sqlx::Result<Vec<Self>> {
        let rows = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_class_student
            WHERE homework_id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    pub async fn count(
        pool: &PgPool,
        student_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> sqlx::Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
        SELECT COUNT(*)
        FROM homework_class_student
        WHERE student_id = $1
          AND created_at >= $2::DATE
          AND created_at < $3::DATE
        "#,
        )
        .bind(student_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn list(
        pool: &PgPool,
        student_id: i64,
        start_date: &str,
        end_date: &str,
        limit: i32,
        offset: i32,
    ) -> sqlx::Result<Vec<Self>> {
        let rows = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_class_student
            WHERE student_id = $1
              AND created_at >= $2::DATE
              AND created_at < $3::DATE
              ORDER BY id DESC
              LIMIT $4 OFFSET $5
            "#,
        )
        .bind(student_id)
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
