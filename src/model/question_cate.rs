use crate::api::question_cate::CreateQuestionCateReq;
use sqlx::{FromRow, PgPool};

/// 题型

#[derive(FromRow)]
pub struct QuestionCate {
    pub id: i32,
    pub related_id: i32,
    pub label: String,
    pub key: String,
    pub sort_order: i32,
}

impl QuestionCate {
    /// 新增记录
    /// 返回生成的完整结构体（包含数据库生成的 ID 和时间戳）
    pub async fn insert(pool: &PgPool, req: CreateQuestionCateReq) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO question_cate (related_id, label, key, sort_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, related_id, label, key, sort_order
            "#,
        )
        .bind(req.related_id)
        .bind(&req.label)
        .bind((&format!("{:x}", md5::compute(&req.label))[..10]).to_string())
        .bind(req.sort_order)
        .fetch_one(pool)
        .await
    }

    /// 根据 ID 删除记录
    pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM question_cate WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    // 通过关联标识获取题型列表
    pub async fn find_all_by_related_ids(
        pool: &PgPool,
        related_ids: Vec<i32>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM question_cate WHERE related_id = ANY($1)")
            .bind(related_ids)
            .fetch_all(pool)
            .await
    }
}
