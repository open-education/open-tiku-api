use chrono::{DateTime, Utc};
use sqlx::FromRow;

// 作业学生测试答题详情表
#[derive(Debug, Clone, FromRow)]
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
    pub async fn save(pool: &sqlx::PgPool, req: Self) -> Result<i64, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            r#"
        INSERT INTO homework_student_test_answer (
            id, attempt_id, question_id, answer,
            result, note, remark, created_at, updated_at
        )
        VALUES (
            COALESCE($1, nextval('homework_student_test_answer_id_seq')),
            $2, $3, $4, $5, $6, $7, $8, $9
        )
        ON CONFLICT (id) DO UPDATE SET
            attempt_id = EXCLUDED.attempt_id,
            question_id = EXCLUDED.question_id,
            answer = EXCLUDED.answer,
            result = EXCLUDED.result,
            note = EXCLUDED.note,
            remark = EXCLUDED.remark,
            created_at = EXCLUDED.created_at,
            updated_at = EXCLUDED.updated_at
        RETURNING id
        "#,
        )
        .bind(req.id)
        .bind(req.attempt_id)
        .bind(req.question_id)
        .bind(req.answer)
        .bind(req.result)
        .bind(req.note)
        .bind(req.remark)
        .bind(req.created_at)
        .bind(req.updated_at)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }
}
