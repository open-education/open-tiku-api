use crate::model::question::{AuthorQuestion, TopTextbook};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

// 合计信息
#[derive(Default, Serialize, Deserialize)]
pub struct CountInfo {
    pub textbook_num: i64,
    pub question_num: i64,
    pub paper_num: i64,
    pub teacher_num: i64,
    pub student_num: i64,
}

#[derive(FromRow, Default)]
pub struct Stat {
    pub date: Option<NaiveDate>,
    pub latest_question_ids: Json<Vec<i64>>,
    pub top_question_ids: Json<Vec<i64>>,
    pub top_textbook_info: Json<Vec<TopTextbook>>,
    pub top_teacher_info: Json<Vec<AuthorQuestion>>,
    pub top_student_info: Json<Vec<i64>>,
    pub count_info: Json<CountInfo>,
}

impl Stat {
    pub async fn save(pool: &PgPool, req: Self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r"
        INSERT INTO stat (date, latest_question_ids, top_question_ids,
                          top_textbook_info, top_teacher_info, top_student_info, count_info)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (date) DO UPDATE SET
            latest_question_ids = EXCLUDED.latest_question_ids,
            top_question_ids    = EXCLUDED.top_question_ids,
            top_textbook_info   = EXCLUDED.top_textbook_info,
            top_teacher_info    = EXCLUDED.top_teacher_info,
            top_student_info    = EXCLUDED.top_student_info,
            count_info          = EXCLUDED.count_info
        ",
        )
        .bind(req.date)
        .bind(&req.latest_question_ids)
        .bind(&req.top_question_ids)
        .bind(&req.top_textbook_info)
        .bind(&req.top_teacher_info)
        .bind(&req.top_student_info)
        .bind(&req.count_info)
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
