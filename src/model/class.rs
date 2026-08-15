use crate::api::class::ClassListReq;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Result};

#[derive(FromRow)]
pub struct Class {
    pub id: Option<i64>,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub label: String,
    pub email: String,
    pub sort_order: i16,
    pub author_id: i64,
    pub remark: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Class {
    pub async fn save(pool: &PgPool, req: Self) -> Result<i64, sqlx::Error> {
        let row_id = sqlx::query(
            r#"
        INSERT INTO class (
            id, year, grade, semester, label, email, sort_order, author_id, remark
        ) VALUES (
            COALESCE($1, nextval('class_id_seq')), $2, $3, $4, $5, $6, $7, $8, $9
        )
        ON CONFLICT (id) DO UPDATE SET
            year = EXCLUDED.year,
            grade = EXCLUDED.grade,
            semester = EXCLUDED.semester,
            label = EXCLUDED.label,
            email = EXCLUDED.email,
            sort_order = EXCLUDED.sort_order,
            author_id = EXCLUDED.author_id,
            remark = EXCLUDED.remark,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
        )
        .bind(req.id)
        .bind(req.year)
        .bind(req.grade)
        .bind(req.semester)
        .bind(req.label)
        .bind(req.email)
        .bind(req.sort_order)
        .bind(req.author_id)
        .bind(req.remark)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            row.get::<i64, _>("id")
        })
        .fetch_one(pool)
        .await?;

        Ok(row_id)
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>> {
        let row = sqlx::query_as::<_, Self>(
            r#"
            SELECT id, year, grade, semester, label, email, author_id, sort_order, remark, created_at, updated_at
            FROM class
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn count(pool: &PgPool, author_id: i64, req: &ClassListReq) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) 
            FROM class
            WHERE author_id = $1
              AND ($2 IS NULL OR year = $2)
              AND ($3 IS NULL OR grade = $3)
              AND ($4 IS NULL OR semester = $4)
            "#,
        )
        .bind(author_id)
        .bind(&req.year)
        .bind(&req.grade)
        .bind(&req.semester)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn list(pool: &PgPool, author_id: i64, req: &ClassListReq) -> Result<Vec<Self>> {
        let offset = (req.page_no - 1) * req.page_size;

        let rows = sqlx::query_as::<_, Self>(
            r#"
            SELECT id, year, grade, semester, label, email, author_id, sort_order, remark, created_at, updated_at
            FROM class
            WHERE author_id = $1
              AND ($2 IS NULL OR year = $2)
              AND ($3 IS NULL OR grade = $3)
              AND ($4 IS NULL OR semester = $4)
            ORDER BY id DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(author_id)
        .bind(&req.year)
        .bind(&req.grade)
        .bind(&req.semester)
        .bind(req.page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
