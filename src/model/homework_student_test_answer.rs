use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};

// 作业学生测试答题详情表
#[derive(FromRow)]
pub struct HomeworkStudentTestAnswer {
    pub id: Option<i64>,
    // 做题记录标识
    pub attempt_id: i64,
    pub question_id: i64,
    // 用户的最终选择/填写内容
    pub answer: String,
    // 是否正确 0 未作答 1 正确 2 错误
    pub result: i16,
    // 笔记
    pub note: String,
    // 备注
    pub remark: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl HomeworkStudentTestAnswer {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        records: &[Self],
    ) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb = sqlx::QueryBuilder::new(
            "INSERT INTO homework_student_test_answer (attempt_id, question_id, answer, result, note, remark) ",
        );

        qb.push_values(records, |mut b, req| {
            b.push_bind(req.attempt_id)
                .push_bind(req.question_id)
                .push_bind(&req.answer)
                .push_bind(req.result)
                .push_bind(&req.note)
                .push_bind(&req.remark);
        });

        Ok(qb.build().execute(&mut **tx).await?.rows_affected())
    }

    pub async fn delete_by_attempt_id(
        tx: &mut Transaction<'_, Postgres>,
        attempt_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let row = sqlx::query(r#"DELETE FROM homework_student_test_answer WHERE attempt_id = $1"#)
            .bind(attempt_id)
            .execute(&mut **tx)
            .await?;

        Ok(row.rows_affected())
    }

    pub async fn find_by_attempt_id(pool: &PgPool, attempt_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM homework_student_test_answer
            WHERE attempt_id = $1
            "#,
        )
        .bind(attempt_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
