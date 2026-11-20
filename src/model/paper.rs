use crate::api::req::paper::PaperAddReq;
use crate::api::req::paper::PaperListReq;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Row, Transaction};

#[derive(FromRow)]
pub struct Paper {
    pub id: Option<i64>,
    pub two_level_id: i32,
    pub five_level_id: Option<i32>,
    pub paper_type: i16,
    pub file_type: i16,
    pub file_list: Option<Json<Vec<FileInfo>>>,
    pub province_id: Option<i32>,
    pub city_id: Option<i32>,
    pub school_name: Option<String>,
    pub exam_type: i16,
    pub year: String,
    pub title: String,
    pub score: Option<i32>,
    pub duration: Option<i16>,
    pub level_type: i16,
    pub difficulty_type: i16,
    pub source: Option<String>,
    pub remark: String,
    pub author_id: i64,
    pub count: Option<i32>,
    pub status: i16,
    pub approve_id: Option<i64>,
    pub reject_reason: Option<String>,
    pub approve_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 文件信息
#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub size: i32,
}

// 统计用户试卷计数
#[derive(FromRow)]
pub struct CountPaper {
    pub status: i16,
    pub count: i64,
}

trait QueryBuilderExt<'a> {
    fn push_list_where(self, req: &'a PaperListReq, author_id: Option<i64>) -> Self;
}

impl<'a> QueryBuilderExt<'a> for QueryBuilder<'a, Postgres> {
    fn push_list_where(mut self, req: &'a PaperListReq, author_id: Option<i64>) -> Self {
        self.push(" WHERE two_level_id = ")
            .push_bind(req.two_level_id);

        if let Some(five_level_id) = req.five_level_id
            && five_level_id > 0
        {
            self.push(" AND five_level_id = ").push_bind(five_level_id);
        }
        if let Some(ref year) = req.year
            && !year.is_empty()
        {
            self.push(" AND year = ").push_bind(year);
        }
        if let Some(exam_type) = req.exam_type
            && exam_type > 0
        {
            self.push(" AND exam_type = ").push_bind(exam_type);
        }
        if let Some(level_type) = req.level_type
            && level_type > 0
        {
            self.push(" AND level_type = ").push_bind(level_type);
        }
        if let Some(difficulty_type) = req.difficulty_type
            && difficulty_type > 0
        {
            self.push(" AND difficulty_type = ")
                .push_bind(difficulty_type);
        }
        if let Some(province_id) = req.province_id
            && province_id > 0
        {
            self.push(" AND province_id = ").push_bind(province_id);
        }
        if let Some(city_id) = req.city_id
            && city_id > 0
        {
            self.push(" AND city_id = ").push_bind(city_id);
        }
        if let Some(ref school_name) = req.school_name
            && !school_name.is_empty()
        {
            self.push(" AND school_name LIKE ")
                .push_bind(format!("%{school_name}%"));
        }
        if let Some(status) = req.status
            && status > 0
        {
            self.push(" AND status = ").push_bind(status);
        }

        // 搜索用户的试卷
        if let Some(uid) = author_id
            && uid > 0
        {
            self.push(" AND author_id = ").push_bind(uid);
        }
        self
    }
}

impl Paper {
    pub async fn save(
        tx: &mut Transaction<'_, Postgres>,
        req: &PaperAddReq,
        author_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            r"
            INSERT INTO paper (
                id, two_level_id, five_level_id, paper_type, file_type,
                file_list, province_id, city_id, school_name, exam_type,
                year, title, score, duration, level_type,
                difficulty_type, source, remark, author_id, count,
                status
            ) VALUES (
                COALESCE($1, nextval(pg_get_serial_sequence('paper', 'id'))),
                $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20,
                $21
            )
            ON CONFLICT (id) DO UPDATE SET
                two_level_id = EXCLUDED.two_level_id,
                five_level_id = EXCLUDED.five_level_id,
                paper_type = EXCLUDED.paper_type,
                file_type = EXCLUDED.file_type,
                file_list = EXCLUDED.file_list,
                province_id = EXCLUDED.province_id,
                city_id = EXCLUDED.city_id,
                school_name = EXCLUDED.school_name,
                exam_type = EXCLUDED.exam_type,
                year = EXCLUDED.year,
                title = EXCLUDED.title,
                score = EXCLUDED.score,
                duration = EXCLUDED.duration,
                level_type = EXCLUDED.level_type,
                difficulty_type = EXCLUDED.difficulty_type,
                source = EXCLUDED.source,
                remark = EXCLUDED.remark,
                count = EXCLUDED.count,
                status = EXCLUDED.status,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            ",
        )
        .bind(req.id)
        .bind(req.two_level_id)
        .bind(req.five_level_id.unwrap_or_default())
        .bind(req.paper_type)
        .bind(req.file_type)
        .bind(Json(req.file_list.as_deref().unwrap_or_default()))
        .bind(req.province_id.unwrap_or_default())
        .bind(req.city_id.unwrap_or_default())
        .bind(req.school_name.as_deref().unwrap_or_default())
        .bind(req.exam_type)
        .bind(&req.year)
        .bind(&req.title)
        .bind(req.score.unwrap_or_default())
        .bind(req.duration.unwrap_or_default())
        .bind(req.level_type)
        .bind(req.difficulty_type)
        .bind(req.source.as_deref().unwrap_or_default())
        .bind(&req.remark)
        .bind(author_id)
        .bind(req.count.unwrap_or_default())
        .bind(req.status)
        .map(|row: sqlx::postgres::PgRow| row.get::<i64, _>("id"))
        .fetch_one(&mut **tx)
        .await?;

        Ok(id)
    }

    pub async fn find_by_id(pool: &PgPool, paper_id: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, Self>(r"SELECT * FROM paper WHERE id = $1")
            .bind(paper_id)
            .fetch_optional(pool)
            .await?;

        Ok(row)
    }

    // 我的题目状态统计
    pub async fn find_count_by_author_id(
        pool: &PgPool,
        author_id: i64,
    ) -> Result<Vec<CountPaper>, sqlx::Error> {
        sqlx::query_as::<_, CountPaper>(
            r"SELECT status, COUNT(*) FROM paper WHERE author_id = $1 GROUP BY status",
        )
        .bind(author_id)
        .fetch_all(pool)
        .await
    }

    pub async fn count(
        pool: &PgPool,
        req: &PaperListReq,
        author_id: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM paper ");

        qb = qb.push_list_where(req, author_id);

        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }

    pub async fn list(
        pool: &PgPool,
        req: &PaperListReq,
        author_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let sql = r"SELECT * FROM paper ";
        let mut qb = QueryBuilder::new(sql);

        qb = qb.push_list_where(req, author_id);

        qb.push(" ORDER BY id DESC LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }
}
