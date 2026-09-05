use crate::enums::test::TestStatus;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

// 作业学生测试尝试记录表
#[derive(FromRow)]
pub struct HomeworkStudentTestAttempt {
    pub id: Option<i64>,
    pub student_id: i64,
    pub homework_id: i64,
    pub class_id: i64,
    pub paper_id: i64,
    // 刷题轮次/批次 第1次刷 第2次刷...
    pub attempt_number: i16,
    // 训练方法 1 练习模式 2 考试模式
    pub method: i16,
    // 状态：1 进行中 2 已交卷
    pub status: i16,
    // 最终总得分 交卷前为0
    pub score: Option<Decimal>,
    // 开始时间
    pub created_at: Option<DateTime<Utc>>,
    // 进度更新时间, 减去开始时间为耗时
    pub updated_at: Option<DateTime<Utc>>,
    // 交卷时间
    pub completed_at: Option<DateTime<Utc>>,
}

impl HomeworkStudentTestAttempt {
    pub async fn save(pool: &PgPool, req: &Self) -> Result<i64, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            r#"
        INSERT INTO homework_student_test_attempt (
            id, student_id, homework_id, class_id, paper_id,
            attempt_number, method, status, score
        )
        VALUES (
            COALESCE($1, nextval('homework_student_test_attempt_id_seq')),
            $2, $3, $4, $5, $6, $7, $8, $9
        )
        ON CONFLICT (id) DO UPDATE SET
            student_id = EXCLUDED.student_id,
            homework_id = EXCLUDED.homework_id,
            class_id = EXCLUDED.class_id,
            paper_id = EXCLUDED.paper_id,
            attempt_number = EXCLUDED.attempt_number,
            method = EXCLUDED.method,
            status = EXCLUDED.status,
            score = EXCLUDED.score,
            created_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
        )
        .bind(req.id)
        .bind(req.student_id)
        .bind(req.homework_id)
        .bind(req.class_id)
        .bind(req.paper_id)
        .bind(req.attempt_number)
        .bind(req.method)
        .bind(req.status)
        .bind(req.score)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }

    pub async fn find_max_attempt_number(
        pool: &PgPool,
        homework_id: i64,
        student_id: i64,
    ) -> Result<Option<i16>, sqlx::Error> {
        let max = sqlx::query_scalar::<_, Option<i16>>(
            "SELECT MAX(attempt_number) FROM homework_student_test_attempt WHERE homework_id = $1 AND student_id = $2",
        )
            .bind(homework_id)
            .bind(student_id)
            .fetch_one(pool)
            .await?;

        Ok(max)
    }

    pub async fn find_in_progress_latest_attempt(
        pool: &PgPool,
        homework_id: i64,
        student_id: i64,
        method: i16,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_student_test_attempt
            WHERE homework_id = $1 AND student_id = $2 AND status = $3 AND method = $4
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(homework_id)
        .bind(student_id)
        .bind(TestStatus::InProgress.as_i16())
        .bind(method)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let row = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_student_test_attempt
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn done_by_id(
        tx: &mut Transaction<'_, Postgres>,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        let row = sqlx::query::<_>(
            r#"
            UPDATE homework_student_test_attempt
            SET
                status = $1,
                updated_at = CURRENT_TIMESTAMP,
                completed_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(TestStatus::Done.as_i16())
        .bind(id)
        .execute(&mut **tx)
        .await?;

        Ok(row.rows_affected())
    }

    pub async fn count(
        pool: &PgPool,
        homework_id: i64,
        student_id: i64,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM homework_student_test_attempt
            WHERE homework_id = $1
              AND student_id = $2
            "#,
        )
        .bind(homework_id)
        .bind(student_id)
        .fetch_one(pool)
        .await
    }

    pub async fn list(
        pool: &PgPool,
        homework_id: i64,
        student_id: i64,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        SELECT *
        FROM homework_student_test_attempt
        WHERE homework_id = $1
          AND student_id = $2
        ORDER BY id DESC
        LIMIT $3 OFFSET $4
        "#,
        )
        .bind(homework_id)
        .bind(student_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}
