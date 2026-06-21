use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

#[derive(FromRow)]
pub struct PaperGroup {
    pub id: i64, // 由业务生成
    pub paper_id: i64,
    pub gen_id: String,
    pub type_name: String,
    pub sub_title: Option<String>,
}

// 试卷题型
impl PaperGroup {
    pub async fn batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        groups: &[Self],
    ) -> Result<(), sqlx::Error> {
        if groups.is_empty() {
            return Ok(());
        }

        let mut builder = QueryBuilder::<Postgres>::new(
            "INSERT INTO paper_group (id, paper_id, gen_id, type_name, sub_title) ",
        );

        builder.push_values(groups, |mut b, group| {
            b.push_bind(group.id)
                .push_bind(group.paper_id)
                .push_bind(&group.gen_id)
                .push_bind(&group.type_name)
                .push_bind(&group.sub_title);
        });

        builder.build().execute(&mut **tx).await?;
        Ok(())
    }

    // 通过 paper_id 查询所有题型（使用 &Pool）
    pub async fn find_by_paper_id(pool: &PgPool, paper_id: i64) -> Result<Vec<Self>, sqlx::Error> {
        let groups = sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM paper_group
            WHERE paper_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(paper_id)
        .fetch_all(pool)
        .await?;

        Ok(groups)
    }
}
