use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};

#[derive(Default, Serialize, Deserialize)]
pub struct TestContent {
    pub id: i16,          // 题目标识自行生成
    pub label: String,    // 题型描述
    pub num: i16,         // 题目数量
    pub score: i16,       // 总分
    pub scores: Vec<i16>, // 每个题的分数, 为空则每个题的分数为 score/num
}

#[derive(FromRow)]
pub struct TestRule {
    pub id: i32,
    pub value: String,
    pub level: i16,
    pub target: String,
    pub score: i16,
    pub content: Json<Vec<TestContent>>,
    pub description: String,
}

impl TestRule {
    // 保存规则
    pub async fn insert(
        pool: &PgPool,
        level: i16,
        target: String,
        score: i16,
        content: Vec<TestContent>,
        description: String,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        INSERT INTO rule (value, level, target, score, content, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        )
        .bind("")
        .bind(level)
        .bind(target)
        .bind(score)
        .bind(Json(content))
        .bind(description)
        .fetch_one(pool)
        .await
    }

    // 获取某个规则详情
    pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM rule WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    // 获取全部规则
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM rule")
            .fetch_all(pool)
            .await
    }
}
