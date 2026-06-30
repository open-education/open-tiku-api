use crate::api::question::CreateQuestionReq;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction, Type};

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
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Content {
    pub content: String,
    pub images: Option<Vec<String>>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Step {
    pub id: i16,         // 步骤顺序
    pub content: String, // 该步骤的内容
}

#[derive(FromRow)]
pub struct Question {
    pub id: i64,
    pub question_cate_id: i32,                          // 题型主键
    pub question_type_id: i32,                          // 题型类型主键
    pub question_tag_ids: Option<Json<Vec<i32>>>,       // 题型标签主键
    pub question_dimension_ids: Option<Json<Vec<i32>>>, // 核心素养标识
    pub author_id: i64,                                 // 作者
    pub source: String,                                 // 来源
    pub original_name: String,                          // 原创者昵称

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
    pub steps: Option<Json<Vec<Step>>>,  // 解题步骤, 学生做题时提示
    pub remark: Option<String>,          // 备注
    pub remark_ext: Option<String>,      // 其它备注

    // 审核相关
    pub status: i16,                       // 审核状态
    pub approve_id: i64,                   // 审核人
    pub reject_reason: Option<String>,     // 拒绝原因
    pub approve_at: Option<DateTime<Utc>>, // 审核时间

    // 创建更新时间
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Question {
    // 添加题目-根据主键判断是新增还是更新
    pub async fn simple_insert(pool: &PgPool, req: CreateQuestionReq) -> Result<i64, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO question (
                id, question_cate_id, question_type_id, question_tag_ids, author_id,
                source, original_name, status,
                title, content_plain, comment, difficulty_level,
                images, options, options_layout,
                answer, knowledge, analysis, process, remark, remark_ext,
                steps, question_dimension_ids
            )
            VALUES (
                COALESCE($1, nextval('question_id_seq')), $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            )
            ON CONFLICT (id) DO UPDATE SET
                (question_cate_id, question_type_id, question_tag_ids, author_id,
                 source, original_name, status,
                 title, content_plain, comment, difficulty_level,
                 images, options, options_layout,
                 answer, knowledge, analysis, process, remark, remark_ext,
                 steps, question_dimension_ids)
                = (EXCLUDED.question_cate_id, EXCLUDED.question_type_id, EXCLUDED.question_tag_ids, EXCLUDED.author_id,
                   EXCLUDED.source, EXCLUDED.original_name, EXCLUDED.status,
                   EXCLUDED.title, EXCLUDED.content_plain, EXCLUDED.comment, EXCLUDED.difficulty_level,
                   EXCLUDED.images, EXCLUDED.options, EXCLUDED.options_layout,
                   EXCLUDED.answer, EXCLUDED.knowledge, EXCLUDED.analysis, EXCLUDED.process, EXCLUDED.remark, EXCLUDED.remark_ext,
                   EXCLUDED.steps, EXCLUDED.question_dimension_ids)
            RETURNING id
        "#,
        )
            .bind(req.id)
            .bind(req.question_cate_id)
            .bind(req.question_type_id)
            .bind(Json(req.question_tag_ids.unwrap_or_default()))
            .bind(req.author_id)
            .bind(req.source)
            .bind(req.original_name)
            .bind(req.status)
            .bind(req.title)
            .bind(req.content_plain)
            .bind(req.comment)
            .bind(req.difficulty_level)
            .bind(Json(req.images.unwrap_or_default()))
            .bind(Json(req.options.unwrap_or_default()))
            .bind(req.options_layout)
            .bind(req.answer)
            .bind(req.knowledge)
            .bind(Json(req.analysis.unwrap_or_default()))
            .bind(Json(req.process.unwrap_or_default()))
            .bind(req.remark)
            .bind(req.remark_ext)
            .bind(Json(req.steps.unwrap_or_default()))
            .bind(Json(req.question_dimension_ids.unwrap_or_default()))
            .fetch_one(pool)
            .await?;

        Ok(id)
    }

    // tx 事务方式写入
    // 注意事务签名类型 tx: &mut Transaction<'_, Postgres>,
    pub async fn tx_insert(
        tx: &mut Transaction<'_, Postgres>,
        req: CreateQuestionReq,
    ) -> Result<Question, sqlx::Error> {
        sqlx::query_as::<_, Question>(
            r#"
        INSERT INTO question (
            question_cate_id, question_type_id, question_tag_ids, author_id,
            source, original_name, status,
            title, content_plain, comment, difficulty_level,
            images, options, options_layout,
            answer, knowledge, analysis, process, remark, remark_ext,
            steps, question_dimension_ids
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
        RETURNING *
        "#,
        )
        .bind(req.question_cate_id)
        .bind(req.question_type_id)
        .bind(Json(req.question_tag_ids.unwrap_or_default()))
        .bind(req.author_id)
        .bind(req.source)
        .bind(req.original_name)
        .bind(req.status)
        .bind(req.title)
        .bind(req.content_plain)
        .bind(req.comment)
        .bind(req.difficulty_level)
        .bind(Json(req.images.unwrap_or_default()))
        .bind(Json(req.options.unwrap_or_default()))
        .bind(req.options_layout)
        .bind(req.answer)
        .bind(req.knowledge)
        .bind(Json(req.analysis.unwrap_or_default()))
        .bind(Json(req.process.unwrap_or_default()))
        .bind(req.remark)
        .bind(req.remark_ext)
        .bind(Json(req.steps.unwrap_or_default()))
        .bind(Json(req.question_dimension_ids.unwrap_or_default()))
        .fetch_one(&mut **tx)
        .await
    }

    // tx 方式批量添加题题目
    // 注意事务签名类型 tx: &mut Transaction<'_, Postgres>,
    pub async fn tx_batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        req_list: Vec<CreateQuestionReq>,
    ) -> Result<Vec<i64>, sqlx::Error> {
        // 请求参数为空则返回空即可
        if req_list.is_empty() {
            return Ok(vec![]);
        }

        // 预分配容量
        let mut all_ids = Vec::with_capacity(req_list.len());

        // 避免 SQL 语句过大
        // 简单看了下一个中等规模的题直接存为 .md 是 1.6k 500*1.6=800k, 大部分题都是选择填空一次性写300条应该暂时没什么风险
        for chunk in req_list.chunks(300) {
            let mut query_builder = QueryBuilder::new(
                r#"
            INSERT INTO question (
                question_cate_id, question_type_id, question_tag_ids, author_id,source,original_name,
                title, content_plain, comment, difficulty_level,
                images, options, options_layout,
                answer, knowledge, analysis, process, remark,remark_ext,
                steps, question_dimension_ids
            )
            "#,
            );

            query_builder.push_values(chunk, |mut b, req| {
                b.push_bind(req.question_cate_id)
                    .push_bind(req.question_type_id)
                    .push_bind(Json(req.question_tag_ids.clone().unwrap_or_default()))
                    .push_bind(req.author_id)
                    .push_bind(&req.source)
                    .push_bind(&req.original_name)
                    .push_bind(&req.title)
                    .push_bind(&req.content_plain)
                    .push_bind(&req.comment)
                    .push_bind(req.difficulty_level)
                    .push_bind(Json(req.images.clone().unwrap_or_default()))
                    .push_bind(Json(req.options.clone().unwrap_or_default()))
                    .push_bind(req.options_layout)
                    .push_bind(&req.answer)
                    .push_bind(&req.knowledge)
                    .push_bind(Json(req.analysis.clone().unwrap_or_default()))
                    .push_bind(Json(req.process.clone().unwrap_or_default()))
                    .push_bind(&req.remark)
                    .push_bind(&req.remark_ext)
                    .push_bind(Json(req.steps.clone().unwrap_or_default()))
                    .push_bind(Json(req.question_dimension_ids.clone().unwrap_or_default()));
            });

            // 添加 RETURNING id 子句
            query_builder.push(" RETURNING id");

            // 执行查询并获取返回的 id 列表
            let ids: Vec<i64> = query_builder
                .build_query_scalar()
                .fetch_all(&mut **tx)
                .await?;

            all_ids.extend(ids);
        }

        Ok(all_ids)
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
        ids: Option<Vec<i64>>,
        title_val: Option<String>,
        tag_ids: Option<Vec<i32>>,
        dimension_ids: Option<Vec<i32>>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM question 
            WHERE question_cate_id = $1
              AND status = $2
              AND ($3 IS NULL OR question_type_id = $3)
              AND ($4 IS NULL OR id = ANY($4))
              AND ($5 IS NULL OR content_plain LIKE '%' || $5 || '%')
              AND ($6 IS NULL OR question_tag_ids @> $7)
              AND ($8 IS NULL OR question_dimension_ids @> $9)
            "#,
        )
        .bind(cate_id)
        .bind(status)
        .bind(type_id)
        .bind(ids)
        .bind(title_val)
        .bind(tag_ids.as_ref().map(|_| true))
        .bind(tag_ids.map(Json))
        .bind(dimension_ids.as_ref().map(|_| true))
        .bind(dimension_ids.map(Json))
        .fetch_one(pool)
        .await
    }

    // 题型下题目列表
    pub async fn list_by_cate_and_type(
        pool: &PgPool,
        cate_id: i32,
        status: i16,
        type_id: Option<i32>,
        ids: Option<Vec<i64>>,
        title_val: Option<String>,
        tag_ids: Option<Vec<i32>>,
        dimension_ids: Option<Vec<i32>>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT *
            FROM question
            WHERE question_cate_id = $1
              AND status = $2
              AND ($3 IS NULL OR question_type_id = $3)
              AND ($4 IS NULL OR id = ANY($4))
              AND ($5 IS NULL OR content_plain LIKE '%' || $5 || '%')
              AND ($6 IS NULL OR question_tag_ids @> $7)
              AND ($8 IS NULL OR question_dimension_ids @> $9)
            ORDER BY id DESC
            LIMIT $10 OFFSET $11
            "#,
        )
        .bind(cate_id)
        .bind(status)
        .bind(type_id)
        .bind(ids)
        .bind(title_val)
        .bind(tag_ids.as_ref().map(|_| true))
        .bind(tag_ids.map(Json))
        .bind(dimension_ids.as_ref().map(|_| true))
        .bind(dimension_ids.map(Json))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
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

    // 母题下面变式题数量
    pub async fn count_similar_by_params(
        pool: &PgPool,
        question_id: i64,
        status: i16,
        cate_id: i32,
        type_id: Option<i32>,
        tag_ids: Option<Vec<i32>>,
        dimension_ids: Option<Vec<i32>>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(q.id)
            FROM question q
            INNER JOIN question_similar qs ON q.id = qs.child_id
            WHERE qs.question_id = $1
              AND q.status = $2
              AND q.question_cate_id = $3
              AND ($4 IS NULL OR q.question_type_id = $4)
              AND ($5 IS NULL OR q.question_tag_ids @> $6)
              AND ($7 IS NULL OR q.question_dimension_ids @> $8)
            "#,
        )
        .bind(question_id)
        .bind(status)
        .bind(cate_id)
        .bind(type_id)
        .bind(tag_ids.as_ref().map(|_| true))
        .bind(tag_ids.map(Json))
        .bind(dimension_ids.as_ref().map(|_| true))
        .bind(dimension_ids.map(Json))
        .fetch_one(pool)
        .await
    }

    // 母题下面变式题列表
    pub async fn list_similar_by_params(
        pool: &PgPool,
        question_id: i64,
        status: i16,
        cate_id: i32,
        type_id: Option<i32>,
        tag_ids: Option<Vec<i32>>,
        dimension_ids: Option<Vec<i32>>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT q.*
            FROM question q
            INNER JOIN question_similar qs ON q.id = qs.child_id
            WHERE qs.question_id = $1
              AND q.status = $2
              AND q.question_cate_id = $3
              AND ($4 IS NULL OR q.question_type_id = $4)
              AND ($5 IS NULL OR q.question_tag_ids @> $6)
              AND ($7 IS NULL OR q.question_dimension_ids @> $8)
            ORDER BY qs.id ASC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(question_id)
        .bind(status)
        .bind(cate_id)
        .bind(type_id)
        .bind(tag_ids.as_ref().map(|_| true))
        .bind(tag_ids.map(Json))
        .bind(dimension_ids.as_ref().map(|_| true))
        .bind(dimension_ids.map(Json))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    /// 根据 ID 删除记录
    pub async fn delete(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM question WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
