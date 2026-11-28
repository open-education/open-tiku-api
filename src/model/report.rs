use crate::api::req::report::{AddReq, ListReq};
use crate::model::paper::FileInfo;
use chrono::{DateTime, Utc};
use sqlx::QueryBuilder;
use sqlx::{FromRow, PgPool, Row, types::Json};

#[derive(FromRow)]
pub struct Report {
    pub id: Option<i64>,
    pub report_type: i16,
    pub report_id: i64,
    pub value: String,
    pub label: String,
    pub label_desc: String,
    pub content: String,
    pub source_url: String,
    pub file_list: Option<Json<Vec<FileInfo>>>,
    pub status: i16,
    pub approve_id: Option<i64>,
    pub reject_reason: Option<String>,
    pub approve_at: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Report {
    pub async fn save(pool: &PgPool, req: &AddReq) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            r"
            INSERT INTO report (
                id, report_type, report_id, value, label,
                label_desc, content, source_url, file_list
            ) VALUES (
                COALESCE($1, nextval(pg_get_serial_sequence('paper', 'id'))),
                $2, $3, $4, $5,
                $6, $7, $8, $9
            )
            ON CONFLICT (id) DO UPDATE SET
                report_type = EXCLUDED.report_type,
                report_id = EXCLUDED.report_id,
                value = EXCLUDED.value,
                label = EXCLUDED.label,
                label_desc = EXCLUDED.label_desc,
                content = EXCLUDED.content,
                source_url = EXCLUDED.source_url,
                file_list = EXCLUDED.file_list,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            ",
        )
        .bind(req.id)
        .bind(req.report_type)
        .bind(req.report_id)
        .bind(&req.value)
        .bind(&req.label)
        .bind(&req.label_desc)
        .bind(&req.content)
        .bind(&req.source_url)
        .bind(Json(req.file_list.as_deref().unwrap_or_default()))
        .map(|row: sqlx::postgres::PgRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    pub async fn count(pool: &PgPool, req: &ListReq) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM report ");

        if let Some(status) = req.status
            && status > 0
        {
            qb.push(" WHERE status = ").push_bind(status);
        }

        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }

    pub async fn list(
        pool: &PgPool,
        req: &ListReq,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<Self>> {
        let sql = r"SELECT * FROM report ";
        let mut qb = QueryBuilder::new(sql);

        if let Some(status) = req.status
            && status > 0
        {
            qb.push(" WHERE status = ").push_bind(status);
        }

        qb.push(" ORDER BY id DESC LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }
}
