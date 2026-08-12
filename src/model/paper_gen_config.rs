use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

// 手动生成试卷配置信息

#[derive(Serialize, Deserialize, Clone)]
pub struct QuestionTypeInfo {
    pub id: i16,
    pub label: String,
    pub num: i16,
    pub score: i16,
}

#[derive(Serialize, Deserialize)]
pub struct DifficultyLevelInfo {
    pub basic: i16,   // 基础题百分比
    pub improve: i16, // 提升题百分比
    pub expand: i16,  // 扩展题百分比
}

#[derive(FromRow)]
pub struct PaperGenConfig {
    pub paper_id: i64,
    pub question_cate_ids: Json<Vec<i32>>,
    pub question_tag_ids: Option<Json<Vec<i16>>>,
    pub question_dimension_ids: Option<Json<Vec<i16>>>,
    pub question_type_info: Json<Vec<QuestionTypeInfo>>,
    pub difficulty_level_info: Json<DifficultyLevelInfo>,
}

impl PaperGenConfig {
    pub async fn tx_insert(
        tx: &mut Transaction<'_, Postgres>,
        conf: &Self,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            r#"
            INSERT INTO paper_gen_config (
                paper_id,
                question_cate_ids,
                question_tag_ids,
                question_dimension_ids,
                question_type_info,
                difficulty_level_info
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(conf.paper_id)
        .bind(&conf.question_cate_ids)
        .bind(&conf.question_tag_ids)
        .bind(&conf.question_dimension_ids)
        .bind(&conf.question_type_info)
        .bind(&conf.difficulty_level_info)
        .map(|row: sqlx::postgres::PgRow| {
            use sqlx::Row;
            row.get::<i64, _>("id")
        })
        .fetch_one(&mut **tx)
        .await?;

        Ok(id)
    }

    pub async fn find_by_paper_id(
        pool: &PgPool,
        paper_id: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        let row =
            sqlx::query_as::<_, Self>(r#"SELECT * FROM paper_gen_config WHERE paper_id = $1"#)
                .bind(paper_id)
                .fetch_optional(pool)
                .await?;
        Ok(row)
    }

    pub async fn delete_by_paper_id(
        tx: &mut Transaction<'_, Postgres>,
        paper_id: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
        DELETE FROM paper_gen_config
        WHERE paper_id = $1
        "#,
            paper_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }
}
