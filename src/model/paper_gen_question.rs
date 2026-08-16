use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

// 手动生成试卷题目信息

#[derive(FromRow)]
pub struct PaperGenQuestion {
    pub id: i64,
    pub paper_id: i64,
    pub group_id: i64,
    pub gen_id: String,
    pub order_num: i16,
    pub question_id: i64,
    pub score: i32,
}

impl PaperGenQuestion {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        items: &[PaperGenQuestion],
    ) -> Result<(), sqlx::Error> {
        if items.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO paper_gen_question (paper_id, group_id, gen_id, order_num, question_id, score) ",
        );

        query_builder.push_values(items, |mut b, item| {
            b.push_bind(item.paper_id)
                .push_bind(item.group_id)
                .push_bind(&item.gen_id)
                .push_bind(item.order_num)
                .push_bind(item.question_id)
                .push_bind(item.score);
        });

        query_builder.build().execute(&mut **tx).await?;

        Ok(())
    }

    pub async fn find_by_group_ids(
        pool: &PgPool,
        paper_id: i64,
        group_ids: &[i64],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if group_ids.is_empty() {
            return Ok(Vec::new());
        }

        let questions = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM paper_gen_question
            WHERE paper_id = $1 AND group_id = ANY($2)
            ORDER BY group_id, id ASC
            "#,
        )
        .bind(paper_id)
        .bind(group_ids)
        .fetch_all(pool)
        .await?;

        Ok(questions)
    }

    pub async fn delete_by_paper_id(
        tx: &mut Transaction<'_, Postgres>,
        paper_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        DELETE FROM paper_gen_question
        WHERE paper_id = $1
        "#,
        )
        .bind(paper_id)
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }
}
