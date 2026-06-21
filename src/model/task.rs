use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};

/// 任务管理

#[derive(FromRow)]
pub struct Task {
    pub id: i64,
    pub question_cate_id: i64,
    pub task_type: i16,
    pub name: String,
    pub url: String,
    pub email: String,
    pub textbook_id: i32,
    pub author_id: i64,
    pub status: i16,
    pub result: Option<String>,
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

impl TaskStatus {
    pub fn desc(code: i16) -> &'static str {
        match code {
            1 => "待处理",
            2 => "处理中",
            3 => "处理成功",
            10 => "处理失败",
            _ => "未知状态",
        }
    }
}

impl Task {
    pub async fn insert(
        pool: &PgPool,
        question_cate_id: i64,
        task_type: i16,
        name: &String,
        author_id: i64,
        url: &String,
        email: &String,
        textbook_id: i32,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
        INSERT INTO task (question_cate_id, task_type, name, url, author_id, status, email, textbook_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        )
            .bind(question_cate_id)
            .bind(task_type)
            .bind(name)
            .bind(url)
            .bind(author_id)
            .bind(TaskStatus::Waiting as i16)
            .bind(email)
            .bind(textbook_id)
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

    pub async fn count_by_cate(
        pool: &PgPool,
        question_cate_id: i64,
        author_id: i64,
        task_type: i16,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM task 
            WHERE question_cate_id=$1 
            AND author_id = $2
              AND task_type = $3
            "#,
        )
        .bind(question_cate_id)
        .bind(author_id)
        .bind(task_type)
        .fetch_one(pool)
        .await
    }

    pub async fn list_by_cate(
        pool: &PgPool,
        question_cate_id: i64,
        author_id: i64,
        task_type: i16,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT *
        FROM task
        WHERE question_cate_id = $1
          AND author_id = $2
          AND task_type = $3
        ORDER BY id DESC
        LIMIT $4 OFFSET $5
        "#,
        )
        .bind(question_cate_id)
        .bind(author_id)
        .bind(task_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    // 所有待执行的任务列表
    pub async fn get_waiting_list(pool: &PgPool, task_type: i16) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT *
        FROM task
        WHERE status = 1
        AND task_type = $1
        "#,
        )
        .bind(task_type)
        .fetch_all(pool)
        .await
    }
}
