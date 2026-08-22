use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

// 作业班级

#[derive(FromRow)]
pub struct HomeworkClass {
    pub batch_no: i32,
    pub homework_id: i64,
    pub paper_id: i64,
    pub class_id: i64,
    pub author_id: i64,
    pub title: String,
    pub remark: String,
}

impl HomeworkClass {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        records: &[Self],
    ) -> Result<u64, sqlx::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut qb = QueryBuilder::new(
            "INSERT INTO homework_class (batch_no, homework_id, class_id, paper_id, author_id, title, remark) ",
        );
        qb.push_values(records, |mut b, r| {
            b.push(r.batch_no)
                .push_bind(r.homework_id)
                .push_bind(r.class_id)
                .push_bind(&r.paper_id)
                .push_bind(r.author_id)
                .push_bind(&r.title)
                .push_bind(&r.remark);
        });
        Ok(qb.build().execute(&mut **tx).await?.rows_affected())
    }

    pub async fn get_max_batch_no(
        pool: &PgPool,
        paper_id: i64,
    ) -> Result<Option<i32>, sqlx::Error> {
        let max = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(batch_no) FROM homework_class WHERE paper_id = $1",
        )
        .bind(paper_id)
        .fetch_one(pool)
        .await?;
        Ok(max)
    }
}
