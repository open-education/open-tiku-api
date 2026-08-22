use sqlx::{FromRow, Postgres, QueryBuilder, Transaction};

#[derive(FromRow)]
pub struct HomeworkClassStudent {
    pub homework_id: i64,
    pub student_id: i64,
}

impl HomeworkClassStudent {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        records: &[Self],
    ) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb =
            QueryBuilder::new("INSERT INTO homework_class_student (homework_id, student_id) ");
        qb.push_values(records, |mut b, r| {
            b.push_bind(r.homework_id).push_bind(r.student_id);
        });
        Ok(qb.build().execute(&mut **tx).await?.rows_affected())
    }
}
