use crate::api::chapter_knowledge::CreateChapterKnowledgeReq;
use sqlx::{FromRow, PgPool};

/// 章节节点和知识点类名称关联关系-目前是一对一的关系

#[derive(FromRow)]
pub struct ChapterKnowledge {
    pub id: i32,
    pub chapter_id: i32,
    pub knowledge_id: i32,
}

impl ChapterKnowledge {
    // 保存关联关系
    pub async fn insert(
        pool: &PgPool,
        req: &CreateChapterKnowledgeReq,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
        INSERT INTO chapter_knowledge (chapter_id, knowledge_id)
        VALUES ($1, $2)
        RETURNING id, chapter_id, knowledge_id
        "#,
        )
        .bind(req.chapter_id)
        .bind(req.knowledge_id)
        .fetch_one(pool)
        .await
    }

    // 更新关联关系
    pub async fn delete_by_chapter_or_knowledge_id(
        pool: &PgPool,
        chapter_or_knowledge_id: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM chapter_knowledge WHERE chapter_id = $1 OR knowledge_id = $1 ",
            chapter_or_knowledge_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    // 通过章节查询已关联的知识点类
    pub async fn find_by_chapter_ids(
        pool: &PgPool,
        chapter_ids: Vec<i32>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM chapter_knowledge WHERE chapter_id = ANY($1)")
            .bind(chapter_ids)
            .fetch_all(pool)
            .await
    }

    // 通过知识点查询关联的章节信息
    pub async fn find_by_knowledge_ids(
        pool: &PgPool,
        knowledge_ids: Vec<i32>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM chapter_knowledge WHERE knowledge_id = ANY($1)")
            .bind(knowledge_ids)
            .fetch_all(pool)
            .await
    }

    // 查看是否已关联
    pub async fn find_unique(
        pool: &PgPool,
        chapter_id: i32,
        knowledge_id: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM chapter_knowledge WHERE chapter_id = $1 AND knowledge_id = $2",
        )
        .bind(chapter_id)
        .bind(knowledge_id)
        .fetch_optional(pool)
        .await
    }

    // 通过章节或者知识点查找关联信息
    pub async fn find_by_chapter_or_knowledge_id(
        pool: &PgPool,
        chapter_or_knowledge_id: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT id, chapter_id, knowledge_id 
     FROM chapter_knowledge 
     WHERE chapter_id = $1 OR knowledge_id = $1 
     LIMIT 1",
        )
        .bind(chapter_or_knowledge_id)
        .fetch_optional(pool)
        .await
    }
}
