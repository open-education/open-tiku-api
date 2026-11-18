use crate::api::req::task::TaskAddReq;
use crate::enums::task::TaskStatus;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

// 任务管理

#[derive(FromRow)]
pub struct Task {
    pub id: i64,
    pub two_level_id: i32,
    pub five_level_id: i32,
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    pub author_id: i64,
    pub status: i16,
    pub result: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    pub async fn insert(
        pool: &PgPool,
        req: TaskAddReq,
        author_id: i64,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r"
        INSERT INTO task (question_cate_id, task_type, name, url, author_id, status, email, two_level_id, five_level_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        ",
        )
            .bind(req.question_cate_id)
            .bind(req.task_type)
            .bind(req.name)
            .bind(req.url)
            .bind(author_id)
            .bind(TaskStatus::Waiting as i16)
            .bind(req.email)
            .bind(req.two_level_id)
            .bind(req.five_level_id)
            .fetch_one(pool)
            .await
    }

    pub async fn update_by_id(
        pool: &PgPool,
        id: &i64,
        status: i16,
        result: String,
    ) -> Result<u64, sqlx::Error> {
        let row = sqlx::query(
            r"
        UPDATE task
        SET status = $2, result = $3, updated_at = NOW()
        WHERE id = $1
        ",
        )
        .bind(id)
        .bind(status)
        .bind(result)
        .execute(pool)
        .await?;

        Ok(row.rows_affected())
    }

    pub async fn count_by_cate(
        pool: &PgPool,
        two_level_id: i32,
        five_level_id: Option<i32>,
        question_cate_id: Option<i64>,
        task_type: i16,
        author_id: i64,
    ) -> Result<i64, sqlx::Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM task WHERE author_id = ");
        qb.push_bind(author_id);

        qb.push(" AND two_level_id = ").push_bind(two_level_id);

        qb.push(" AND task_type = ").push_bind(task_type);

        // 只有当参数为 Some 时，才拼接该条件，保证索引高效命中
        if let Some(five_id) = five_level_id
            && five_id > 0
        {
            qb.push(" AND five_level_id = ").push_bind(five_id);
        }
        if let Some(cate_id) = question_cate_id
            && cate_id > 0
        {
            qb.push(" AND question_cate_id = ").push_bind(cate_id);
        }

        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }

    pub async fn list_by_cate(
        pool: &PgPool,
        two_level_id: i32,
        five_level_id: Option<i32>,
        question_cate_id: Option<i64>,
        task_type: i16,
        author_id: i64,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM task WHERE author_id = ");
        qb.push_bind(author_id);

        qb.push(" AND two_level_id = ").push_bind(two_level_id);

        qb.push(" AND task_type = ").push_bind(task_type);

        // 动态拼接可选过滤条件
        if let Some(five_id) = five_level_id
            && five_id > 0
        {
            qb.push(" AND five_level_id = ").push_bind(five_id);
        }
        if let Some(cate_id) = question_cate_id
            && cate_id > 0
        {
            qb.push(" AND question_cate_id = ").push_bind(cate_id);
        }

        qb.push(" ORDER BY id DESC LIMIT ");
        qb.push_bind(limit);

        qb.push(" OFFSET ");
        qb.push_bind(offset);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }

    // 所有待执行的任务列表
    pub async fn get_waiting_list(pool: &PgPool, task_type: i16) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r"
        SELECT *
        FROM task
        WHERE status = 1
        AND task_type = $1
        ",
        )
        .bind(task_type)
        .fetch_all(pool)
        .await
    }
}
