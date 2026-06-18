use crate::model::question::{Content, QuestionOption};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

#[derive(FromRow)]
pub struct PaperQuestion {
    pub id: i64,
    pub paper_id: i64,
    pub group_id: i64, // 由业务生成
    pub gen_id: String,
    pub stem: String,
    pub images: Option<Json<Vec<String>>>,
    pub options: Option<Json<Vec<QuestionOption>>>,
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
            "INSERT INTO paper_question (paper_id, group_id, gen_id, stem, images, options, answer, analysis, score) ",
        );

        builder.push_values(questions, |mut b, q| {
            b.push_bind(&q.paper_id)
                .push_bind(q.group_id)
                .push_bind(&q.gen_id)
                .push_bind(&q.stem)
                .push_bind(&q.images)
                .push_bind(&q.options)
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
            SELECT 
                id,
                paper_id,
                group_id,
                gen_id,
                stem,
                images, 
                options,
                answer,
                analysis,
                score
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
}
