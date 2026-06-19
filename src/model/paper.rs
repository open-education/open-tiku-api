use crate::api::paper::PaperListReq;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction, Type, query_as, query_scalar};

/// 试卷相关

#[derive(FromRow)]
pub struct Paper {
    pub id: i64,
    pub related_id: i32,
    pub related_name: String,
    pub tag: String,
    pub year: String,
    pub grade: String,
    pub semester: String,
    pub title: String,
    pub score: i32,
    pub source: String,
    pub remark: Option<String>,
    pub author_id: i64,
    pub author_name: String,
    pub count: i32,
    pub remark_ext: Option<String>,
    pub status: i16,
    pub approve_id: i64,
    pub reject_reason: Option<String>,
    pub approve_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum PaperStatus {
    Draft = 0,     // 0: 草稿
    Pending = 1,   // 1: 待审核
    Published = 2, // 2: 已发布
    Rejected = 3,  // 3: 被拒绝
}

impl PaperStatus {
    pub fn desc(code: i16) -> String {
        match code {
            0 => "草稿".to_string(),
            1 => "待审核".to_string(),
            2 => "已发布".to_string(),
            3 => "被拒绝".to_string(),
            _ => "未知状态".to_string(),
        }
    }
}

// 试卷主表
impl Paper {
    pub async fn insert(
        tx: &mut Transaction<'_, Postgres>,
        paper: &Self,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO paper (
                related_id, related_name, tag, year, grade, semester,
                title, score, source, remark, author_id, author_name,
                count, remark_ext, status, approve_id, reject_reason, approve_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18
            )
            RETURNING id
            "#,
        )
        .bind(paper.related_id)
        .bind(&paper.related_name)
        .bind(&paper.tag)
        .bind(&paper.year)
        .bind(&paper.grade)
        .bind(&paper.semester)
        .bind(&paper.title)
        .bind(paper.score)
        .bind(&paper.source)
        .bind(&paper.remark)
        .bind(paper.author_id)
        .bind(&paper.author_name)
        .bind(paper.count)
        .bind(&paper.remark_ext)
        .bind(paper.status)
        .bind(paper.approve_id)
        .bind(&paper.reject_reason)
        .bind(paper.approve_at)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            row.get::<i64, _>("id")
        })
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    // 通过主键 id 查询单个试卷（使用 &Pool）
    pub async fn find_by_id(pool: &PgPool, paper_id: i64) -> Result<Option<Self>, sqlx::Error> {
        let paper = sqlx::query_as::<_, Self>(
            r#"
            SELECT 
                id,
                related_id,
                related_name,
                tag,
                year,
                grade,
                semester,
                title,
                score,
                source,
                remark,
                author_id,
                author_name,
                count,
                remark_ext,
                status,
                approve_id,
                reject_reason,
                approve_at,
                created_at,
                updated_at
            FROM paper
            WHERE id = $1
            "#,
        )
        .bind(paper_id)
        .fetch_optional(pool)
        .await?;

        Ok(paper)
    }

    /// 构建 WHERE 子句，返回 (where_clause, param_count)
    /// 参数顺序固定：related_id（必填），tag（可选），year（可选），grade（可选），semester（可选）
    pub fn build_condition(req: &PaperListReq) -> (String, usize) {
        let mut conditions = Vec::new();
        let mut param_count = 0;

        // related_id 必填，占位符 $1
        conditions.push(format!("related_id = ${}", param_count + 1));
        param_count += 1;

        if let Some(_) = &req.tag {
            param_count += 1;
            conditions.push(format!("tag = ${}", param_count));
        }
        if let Some(_) = &req.year {
            param_count += 1;
            conditions.push(format!("year = ${}", param_count));
        }
        if let Some(_) = &req.grade {
            param_count += 1;
            conditions.push(format!("grade = ${}", param_count));
        }
        if let Some(_) = &req.semester {
            param_count += 1;
            conditions.push(format!("semester = ${}", param_count));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, param_count)
    }

    /// 查询总数，需要传入 where_clause 和 param_count（由 build_filter 返回）
    pub async fn count(
        pool: &PgPool,
        req: &PaperListReq,
        where_clause: &str,
    ) -> Result<i64, sqlx::Error> {
        let sql = format!("SELECT COUNT(*) FROM paper {}", where_clause);
        let mut query = query_scalar::<_, i64>(&sql);

        // 按固定顺序绑定参数（与 build_filter 中的占位符顺序一致）
        query = query.bind(req.related_id);
        if let Some(tag) = &req.tag {
            query = query.bind(tag);
        }
        if let Some(year) = &req.year {
            query = query.bind(year);
        }
        if let Some(grade) = &req.grade {
            query = query.bind(grade);
        }
        if let Some(semester) = &req.semester {
            query = query.bind(semester);
        }

        let total = query.fetch_one(pool).await?;
        Ok(total)
    }

    /// 查询分页列表，需要传入 where_clause 和 param_count（由 build_filter 返回）
    pub async fn list(
        pool: &PgPool,
        req: &PaperListReq,
        where_clause: &str,
        param_count: usize,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = (req.page_no - 1) * req.page_size;
        // LIMIT 和 OFFSET 占位符为 param_count+1 和 param_count+2
        let sql = format!(
            "SELECT * FROM paper {} ORDER BY id LIMIT ${} OFFSET ${}",
            where_clause,
            param_count + 1,
            param_count + 2
        );

        let mut query = query_as::<_, Paper>(&sql);

        // 绑定过滤参数（顺序与 count_total 完全一致）
        query = query.bind(req.related_id);
        if let Some(tag) = &req.tag {
            query = query.bind(tag);
        }
        if let Some(year) = &req.year {
            query = query.bind(year);
        }
        if let Some(grade) = &req.grade {
            query = query.bind(grade);
        }
        if let Some(semester) = &req.semester {
            query = query.bind(semester);
        }

        // 绑定 LIMIT 和 OFFSET
        query = query.bind(req.page_size);
        query = query.bind(offset);

        let papers = query.fetch_all(pool).await?;
        Ok(papers)
    }

    // 获取最新的部分试卷
    pub async fn get_latest_papers(
        pool: &PgPool,
        limit: i64, // 传入需要获取的条数
    ) -> Result<Vec<Self>, sqlx::Error> {
        // 若 limit <= 0，直接返回空向量（或视业务需求抛错）
        if limit <= 0 {
            return Ok(vec![]);
        }

        let papers = sqlx::query_as::<_, Self>("SELECT * FROM paper ORDER BY id DESC LIMIT $1")
            .bind(limit) // 绑定参数
            .fetch_all(pool)
            .await?;

        Ok(papers)
    }
}
