use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

/// 任务管理

#[derive(FromRow)]
pub struct Task {
    pub id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    pub author_id: i64,
    pub status: i16,
    pub result: String,
    // 创建更新时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum TaskType {
    UploadQuestion = 1, // 题目上传
}

#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum TaskStatus {
    Waiting = 1, // 待处理
    Running = 2, // 处理中
    Success = 3, // 处理成功
    Failed = 10, // 处理失败
}

impl Task {
    pub async fn insert(
        pool: &PgPool,
        task_type: i16,
        name: &String,
        author_id: i64,
        url: &String,
        email: &String,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO task (task_type, name, url, author_id, status, email)
                VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(task_type)
        .bind(name)
        .bind(url)
        .bind(author_id)
        .bind(TaskStatus::Waiting)
        .bind(email)
        .fetch_one(pool)
        .await
    }

    pub async fn update_by_id(
        pool: &PgPool,
        id: &i64,
        status: i16,
        result: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE task
        SET status = $2, result = $3, updated_at = NOW()
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(status)
        .bind(result)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    pub async fn count_by_author(
        pool: &PgPool,
        author_id: i64,
        task_type: i16,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM task 
            WHERE author_id = $1
              AND task_type = $2
            "#,
        )
        .bind(author_id)
        .bind(task_type)
        .fetch_one(pool)
        .await
    }

    pub async fn list_by_author(
        pool: &PgPool,
        author_id: i64,
        task_type: i16,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT 
                id, author_id, task_type, name, email, url, author_id, status, result, created_at, updated_at
            FROM task
            WHERE author_id = $1
              AND task_type = $2
            ORDER BY id DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(author_id)
        .bind(task_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}
