use crate::model::question::{Content, QuestionOption};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

#[derive(FromRow)]
pub struct PaperQuestion {
    pub id: i64,
    pub paper_id: i64,
    pub group_id: i64,
    pub gen_id: String,
    pub order_num: i16,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
    pub options_layout: Option<i16>,
    pub answer: Option<String>,
    pub analysis: Option<Json<Content>>,
    pub score: i32,
}

// 试卷题目
impl PaperQuestion {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        questions: &[Self],
    ) -> Result<(), sqlx::Error> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "INSERT INTO paper_question (paper_id, group_id, gen_id, order_num, stem, images, options, options_layout, answer, analysis, score) ",
        );

        builder.push_values(questions, |mut b, q| {
            b.push_bind(&q.paper_id)
                .push_bind(q.group_id)
                .push_bind(&q.gen_id)
                .push_bind(&q.order_num)
                .push_bind(&q.stem)
                .push_bind(&q.images)
                .push_bind(&q.options)
                .push_bind(&q.options_layout)
                .push_bind(&q.answer)
                .push_bind(&q.analysis)
                .push_bind(q.score);
        });

        builder.build().execute(&mut **tx).await?;
        Ok(())
    }

    // 通过 group_id 列表查询所有题目（使用 &Pool）
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
            FROM paper_question
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

    // 根据 paper_id 删除所有题目
    pub async fn delete_by_paper_id(
        tx: &mut Transaction<'_, Postgres>,
        paper_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
        DELETE FROM paper_question
        WHERE paper_id = $1
        "#,
            paper_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }
}
