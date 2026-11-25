use crate::api::req::paper::{PaperAddReq, PaperListReq};
use crate::enums::paper::PaperStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Row, Transaction};

#[derive(FromRow, Clone)]
pub struct Paper {
    pub id: Option<i64>,
    pub textbook_path: String,
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
#[derive(Serialize, Deserialize, Clone)]
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

// 更新状态对象
pub struct UpdatePaperReq {
    pub id: i64,
    pub status: i16,
    pub approve_id: i64,
    pub reject_reason: Option<String>,
    pub approve_at: Option<DateTime<Utc>>,
}

trait QueryBuilderExt<'a> {
    fn push_list_where(self, req: &'a PaperListReq, author_id: Option<i64>) -> Self;
}

impl<'a> QueryBuilderExt<'a> for QueryBuilder<'a, Postgres> {
    fn push_list_where(mut self, req: &'a PaperListReq, author_id: Option<i64>) -> Self {
        self.push(" WHERE textbook_path LIKE ")
            .push_bind(format!("{}%", req.textbook_path));

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
                id, textbook_path, paper_type, file_type, file_list,
                province_id, city_id, school_name, exam_type, year,
                title, score, duration, level_type, difficulty_type,
                source, remark, author_id, count, status
            ) VALUES (
                COALESCE($1, nextval(pg_get_serial_sequence('paper', 'id'))),
                $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20
            )
            ON CONFLICT (id) DO UPDATE SET
                textbook_path = EXCLUDED.textbook_path,
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
        .bind(&req.textbook_path)
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

    pub async fn find_by_ids(pool: &PgPool, ids: Vec<i64>) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Self>(r"SELECT * FROM paper WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await?;

        Ok(rows)
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

    // 更新状态
    pub async fn update_status_by_id(
        pool: &PgPool,
        req: UpdatePaperReq,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r"
        UPDATE paper
        SET status = $2,
            approve_id = $3,
            reject_reason = $4,
            approve_at = $5,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        ",
        )
        .bind(req.id)
        .bind(req.status)
        .bind(req.approve_id)
        .bind(req.reject_reason)
        .bind(req.approve_at)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    // 根据 ID 删除记录
    pub async fn delete(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
        let row = sqlx::query("DELETE FROM paper WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(row.rows_affected())
    }

    // 审核通过的试卷套数
    // task cron 统计用其它方法不要使用, 可能没有针对性创建索引存在性能问题
    pub async fn find_paper_num(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(r"SELECT COUNT(*) FROM paper WHERE status = $1")
            .bind(PaperStatus::Published as i16)
            .fetch_one(pool)
            .await
    }
}
