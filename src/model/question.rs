use crate::api::question::CreateQuestionReq;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Type};

/// 题目

// 选项内容
#[derive(Serialize, Deserialize, Clone)]
pub struct QuestionOption {
    pub label: String,               // A, B, C, D, E
    pub content: String,             // 选项内容
    pub images: Option<Vec<String>>, // 图片列表
    pub order: i32,                  // 顺序
}

// 审核状态枚举
#[derive(Serialize, Deserialize, Type, PartialEq)]
#[repr(i16)]
pub enum QuestionStatus {
    Draft = 0,     // 0: 草稿
    Pending = 1,   // 1: 待审核
    Published = 2, // 2: 已发布
    Rejected = 3,  // 3: 被拒绝
}

// 解题分析
#[derive(Default, Serialize, Deserialize)]
pub struct Content {
    pub content: String,
    pub images: Option<Vec<String>>,
}

#[derive(FromRow)]
pub struct Question {
    pub id: i64,
    pub question_cate_id: i32,                    // 题型主键
    pub question_type_id: i32,                    // 题型类型主键
    pub question_tag_ids: Option<Json<Vec<i32>>>, // 题型标签主键
    pub author_id: i64,                           // 作者

    pub title: String,           // 标题
    pub content_plain: String,   // 去除公式等特殊字符的标题, 为了搜索用
    pub comment: Option<String>, // 标题补充说明

    // 使用 rust_decimal 处理 0.5 精度问题
    pub difficulty_level: Decimal, // 题目难易程度

    pub images: Option<Json<Vec<String>>>, // 题目图片列表

    pub options: Option<Json<Vec<QuestionOption>>>, // 选项内容
    pub options_layout: Option<i16>,                // 使用 i16 对应数据库 SMALLINT

    // 答案与解析
    pub answer: Option<String>,          // 参考答案
    pub knowledge: Option<String>,       // 知识点文本描述
    pub analysis: Option<Json<Content>>, // 解题分析
    pub process: Option<Json<Content>>,  // 解题过程
    pub remark: Option<String>,          // 备注

    // 审核相关
    pub status: i16,                                       // 审核状态
    pub approve_id: i64,                                   // 审核人
    pub reject_reason: Option<String>,                     // 拒绝原因
    pub approve_at: Option<chrono::DateTime<chrono::Utc>>, // 审核时间

    // 创建更新时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Question {
    // 添加题目
    pub async fn insert(pool: &PgPool, req: CreateQuestionReq) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO question (
                question_cate_id, question_type_id, question_tag_ids, author_id,
                title, content_plain, comment, difficulty_level, 
                images, options, options_layout, 
                answer, knowledge, analysis, process, remark
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING 
                id, question_cate_id, question_type_id, question_tag_ids, author_id, 
                title, content_plain, comment, difficulty_level, 
                images, options, options_layout, 
                answer, knowledge, analysis, process, remark,
                status, approve_id, reject_reason, approve_at,
                created_at, updated_at
            "#,
        )
        .bind(req.question_cate_id)
        .bind(req.question_type_id)
        .bind(Json(req.question_tag_ids.unwrap_or_default()))
        .bind(req.author_id)
        .bind(req.title)
        .bind(req.content_plain)
        .bind(req.comment)
        .bind(req.difficulty_level)
        .bind(Json(req.images.unwrap_or_default())) // 确保 JSONB 字段正确包装, 不提供默认值会存入 NULL
        .bind(Json(req.options.unwrap_or_default()))
        .bind(req.options_layout) // 显式转为 i16 对应 SMALLINT
        .bind(req.answer)
        .bind(req.knowledge)
        .bind(Json(req.analysis.unwrap_or_default()))
        .bind(Json(req.process.unwrap_or_default()))
        .bind(req.remark)
        .fetch_one(pool)
        .await
    }

    // 通过id获取详情
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM question WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    // 题型下题目数量
    pub async fn count_by_cate_and_type(
        pool: &PgPool,
        cate_id: i32,
        status: i16,
        type_id: Option<i32>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM question 
            WHERE question_cate_id = $1
              AND status = $2
              AND ($3 IS NULL OR question_type_id = $3)
            "#,
        )
        .bind(cate_id)
        .bind(status)
        .bind(type_id)
        .fetch_one(pool)
        .await
    }

    // 题型下题目列表
    pub async fn list_by_cate_and_type(
        pool: &PgPool,
        cate_id: i32,
        status: i16,
        type_id: Option<i32>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT 
                id, question_cate_id, question_type_id, question_tag_ids, author_id,
                title, content_plain, comment, difficulty_level, 
                images, options, options_layout, 
                answer, knowledge, analysis, process, remark,
                status, approve_id, reject_reason, approve_at,
                created_at, updated_at
            FROM question
            WHERE question_cate_id = $1
              AND status = $2
              AND ($3 IS NULL OR question_type_id = $3)
            ORDER BY id DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(cate_id)
        .bind(status)
        .bind(type_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    // 更新题目类型
    pub async fn update_question_type_by_id(
        pool: &PgPool,
        id: i64,
        question_type_id: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET question_type_id = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(question_type_id)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新题目标签
    pub async fn update_question_tags_by_id(
        pool: &PgPool,
        id: i64,
        question_tag_ids: Vec<i32>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET question_tag_ids = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Json(question_tag_ids))
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新题目难易程度
    pub async fn update_difficulty_level_by_id(
        pool: &PgPool,
        id: i64,
        difficulty_level: Decimal,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET difficulty_level = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(difficulty_level)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新标题
    pub async fn update_title_by_id(
        pool: &PgPool,
        id: i64,
        title: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET title = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(title)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新标题补充说明
    pub async fn update_comment_by_id(
        pool: &PgPool,
        id: i64,
        comment: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET comment = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(comment)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新标题图片列表
    pub async fn update_images_by_id(
        pool: &PgPool,
        id: i64,
        images: Vec<String>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET images = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Json(images))
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新选项样式
    pub async fn update_options_layout_by_id(
        pool: &PgPool,
        id: i64,
        options_layout: i16,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET options_layout = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(options_layout)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新选项
    pub async fn update_options_by_id(
        pool: &PgPool,
        id: i64,
        options: Vec<QuestionOption>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET options = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Json(options))
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新参考答案
    pub async fn update_answer_by_id(
        pool: &PgPool,
        id: i64,
        answer: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET answer = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(answer)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新知识点
    pub async fn update_knowledge_by_id(
        pool: &PgPool,
        id: i64,
        knowledge: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET knowledge = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(knowledge)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新解题分析
    pub async fn update_analysis_by_id(
        pool: &PgPool,
        id: i64,
        analysis: Content,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET analysis = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Json(analysis))
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新解题过程
    pub async fn update_process_by_id(
        pool: &PgPool,
        id: i64,
        process: Content,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET process = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(Json(process))
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 更新备注
    pub async fn update_remark_by_id(
        pool: &PgPool,
        id: i64,
        remark: String,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
        UPDATE question
        SET remark = $2
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(remark)
        .execute(pool)
        .await?;

        // rows_affected() 返回受影响的行数
        Ok(result.rows_affected())
    }

    // 题型下是否存在题目
    pub async fn exist_by_cate_id(pool: &PgPool, cate_id: i32) -> Result<bool, sqlx::Error> {
        // EXISTS 返回布尔值
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
        SELECT EXISTS(SELECT 1 FROM question WHERE question_cate_id = $1)
        "#,
        )
        .bind(cate_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }

    // 更新状态
    pub async fn update_status_by_id(
        pool: &PgPool,
        id: i64,
        status: i16,
        approve_id: i64,
        reject_reason: Option<String>,
    ) -> Result<u64, sqlx::Error> {
        // 1. 获取当前 UTC 时间用于更新 approve_at
        let now = Utc::now();

        let result = sqlx::query(
            r#"
        UPDATE question
        SET status = $2, 
            approve_id = $3, 
            reject_reason = $4, 
            approve_at = $5
        WHERE id = $1
        "#,
        )
        .bind(id)
        .bind(status)
        .bind(approve_id)
        .bind(reject_reason)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
