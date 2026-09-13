use chrono::NaiveDate;
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Default)]
pub struct Stat {
    pub date: Option<NaiveDate>,
    pub latest_question_ids: Json<Vec<i64>>,
    pub top_question_ids: Json<Vec<i64>>,
    pub top_textbook_ids: Json<Vec<i32>>,
    pub top_teacher_ids: Json<Vec<i64>>,
    pub top_student_ids: Json<Vec<i64>>,
}

impl Stat {
    pub async fn save(pool: &PgPool, req: Self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        INSERT INTO stat (date, latest_question_ids, top_question_ids,
                          top_textbook_ids, top_teacher_ids, top_student_ids)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (date) DO UPDATE SET
            latest_question_ids = EXCLUDED.latest_question_ids,
            top_question_ids    = EXCLUDED.top_question_ids,
            top_textbook_ids    = EXCLUDED.top_textbook_ids,
            top_teacher_ids     = EXCLUDED.top_teacher_ids,
            top_student_ids     = EXCLUDED.top_student_ids
        "#,
        )
        .bind(req.date)
        .bind(&req.latest_question_ids)
        .bind(&req.top_question_ids)
        .bind(&req.top_textbook_ids)
        .bind(&req.top_teacher_ids)
        .bind(&req.top_student_ids)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, date: NaiveDate) -> sqlx::Result<Option<Self>> {
        let row = sqlx::query_as::<_, Self>(r"SELECT * FROM stat WHERE date = $1 ")
            .bind(date)
            .fetch_optional(pool)
            .await?;

        Ok(row)
    }
}
