use crate::api::req::paper::GenPaperGenConfig;
use crate::api::req::question::CreateQuestionReq;
use crate::enums::dict::TypeCode;
use crate::enums::question::{QuestionRelationType, QuestionStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Transaction};

// 题目

// 选项内容
#[derive(Serialize, Deserialize, Clone)]
pub struct QuestionOption {
    pub label: String,               // A, B, C, D, E
    pub content: String,             // 选项内容
    pub images: Option<Vec<String>>, // 图片列表
    pub order: i32,                  // 顺序
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
    pub level_id: i32,                                  // 分层体系
    pub scene_ids: Option<Json<Vec<i32>>>,              // 适用场景
    pub mistake_tip_ids: Option<Json<Vec<i32>>>,        // 常见错误
    pub relation_type: i16,                             // 题目类型
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
    #[sqlx(default)]
    pub answer: Option<String>, // 参考答案
    #[sqlx(default)]
    pub knowledge: Option<String>, // 知识点文本描述
    #[sqlx(default)]
    pub analysis: Option<Json<Content>>, // 解题分析
    #[sqlx(default)]
    pub process: Option<Json<Content>>, // 解题过程
    pub steps: Option<Json<Vec<Step>>>, // 解题步骤, 学生做题时提示
    #[sqlx(default)]
    pub remark: Option<String>, // 备注

    // 审核相关
    pub status: i16,                       // 审核状态
    pub approve_id: i64,                   // 审核人
    pub reject_reason: Option<String>,     // 拒绝原因
    pub approve_at: Option<DateTime<Utc>>, // 审核时间

    // 创建更新时间
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 普通题目列表请求
pub struct CateAndTypeReq {
    pub cate_ids: Vec<i32>,
    pub status: i16,
    pub type_id: Option<i32>,
    pub ids: Option<Vec<i64>>,
    pub title_val: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
    pub dimension_ids: Option<Vec<i32>>,
    pub author_id: Option<i64>,
    pub level_ids: Option<Vec<i32>>,
    pub scene_ids: Option<Vec<i32>>,
    pub mistake_tip_ids: Option<Vec<i32>>,
}

#[derive(Default)]
pub struct ExtIdReq {
    pub type_id: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
    pub dimension_ids: Option<Vec<i32>>,
    pub level_id: Option<i32>,
    pub scene_ids: Option<Vec<i32>>,
    pub mistake_tip_ids: Option<Vec<i32>>,
}

impl ExtIdReq {
    pub fn from_type_code(type_code: &TypeCode, row_id: i32) -> Self {
        match type_code {
            TypeCode::Type => Self {
                type_id: Some(row_id),
                ..Default::default()
            },
            TypeCode::Tag => Self {
                tag_ids: Some(vec![row_id]),
                ..Default::default()
            },
            TypeCode::Dimension => Self {
                dimension_ids: Some(vec![row_id]),
                ..Default::default()
            },
            TypeCode::Level => Self {
                level_id: Some(row_id),
                ..Default::default()
            },
            TypeCode::Scene => Self {
                scene_ids: Some(vec![row_id]),
                ..Default::default()
            },
            TypeCode::MistakeTip => Self {
                mistake_tip_ids: Some(vec![row_id]),
                ..Default::default()
            },
        }
    }
}

// 变式题列表请求
pub struct SimilarReq {
    pub question_id: i64,
    pub status: i16,
    pub cate_id: i32,
    pub type_id: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
    pub dimension_ids: Option<Vec<i32>>,
}

trait QueryBuilderExt<'a> {
    fn push_cate_and_type_where(self, req: &'a CateAndTypeReq) -> Self;
    fn push_similar_where(self, req: &'a SimilarReq) -> Self;
    fn push_random_where(self, req: &'a GenPaperGenConfig) -> Self;
}

// 为 sqlx::QueryBuilder 实现这个 Trait
impl<'a> QueryBuilderExt<'a> for QueryBuilder<'a, Postgres> {
    fn push_cate_and_type_where(mut self, req: &'a CateAndTypeReq) -> Self {
        self.push(" WHERE question_cate_id = ANY(")
            .push_bind(&req.cate_ids)
            .push(")");
        self.push(" AND status = ").push_bind(req.status);

        if let Some(type_id) = req.type_id
            && type_id > 0
        {
            self.push(" AND question_type_id = ").push_bind(type_id);
        }
        if let Some(ref ids) = req.ids
            && !ids.is_empty()
        {
            self.push(" AND id = ANY(").push_bind(ids).push(")");
        }
        if let Some(ref title_val) = req.title_val
            && !title_val.is_empty()
        {
            self.push(" AND content_plain LIKE ")
                .push_bind(format!("%{title_val}%"));
        }
        if let Some(ref tag_ids) = req.tag_ids
            && !tag_ids.is_empty()
        {
            self.push(" AND question_tag_ids @> ")
                .push_bind(Json(tag_ids));
        }
        if let Some(ref dimension_ids) = req.dimension_ids
            && !dimension_ids.is_empty()
        {
            self.push(" AND question_dimension_ids @> ")
                .push_bind(Json(dimension_ids));
        }
        if let Some(author_id) = req.author_id
            && author_id > 0
        {
            self.push(" AND author_id = ").push_bind(author_id);
        }
        if let Some(ref level_ids) = req.level_ids
            && !level_ids.is_empty()
        {
            self.push(" AND level_id = ANY(")
                .push_bind(level_ids)
                .push(")");
        }
        if let Some(ref scene_ids) = req.scene_ids
            && !scene_ids.is_empty()
        {
            self.push(" AND scene_ids @> ").push_bind(Json(scene_ids));
        }
        if let Some(ref mistake_tip_ids) = req.mistake_tip_ids
            && !mistake_tip_ids.is_empty()
        {
            self.push(" AND mistake_tip_ids @> ")
                .push_bind(Json(mistake_tip_ids));
        }
        self
    }

    fn push_similar_where(mut self, req: &'a SimilarReq) -> Self {
        self.push(" WHERE qs.question_id = ")
            .push_bind(req.question_id);
        self.push(" AND q.status = ").push_bind(req.status);
        self.push(" AND q.question_cate_id = ")
            .push_bind(req.cate_id);
        self.push(" AND qs.question_type = ")
            .push_bind(QuestionRelationType::Similar as i16);
        if let Some(type_id) = req.type_id
            && type_id > 0
        {
            self.push(" AND q.question_type_id = ").push_bind(type_id);
        }
        if let Some(ref tag_ids) = req.tag_ids
            && !tag_ids.is_empty()
        {
            self.push(" AND q.question_tag_ids @> ")
                .push_bind(Json(tag_ids));
        }
        if let Some(ref dimension_ids) = req.dimension_ids
            && !dimension_ids.is_empty()
        {
            self.push(" AND q.question_dimension_ids @> ")
                .push_bind(Json(dimension_ids));
        }
        self
    }

    fn push_random_where(mut self, req: &'a GenPaperGenConfig) -> Self {
        self.push(" AND question_cate_id = ANY(")
            .push_bind(&req.question_cate_ids)
            .push(")");
        self.push(" AND status = ")
            .push_bind(QuestionStatus::Published as i16);

        if let Some(ref tags) = req.tag_ids
            && !tags.is_empty()
        {
            self.push(" AND question_tag_ids @> ").push_bind(Json(tags));
        }
        if let Some(ref dimensions) = req.dimension_ids
            && !dimensions.is_empty()
        {
            self.push(" AND question_dimension_ids @> ")
                .push_bind(Json(dimensions));
        }
        if let Some(ref level_ids) = req.level_ids
            && !level_ids.is_empty()
        {
            self.push(" AND level_id = ANY(")
                .push_bind(level_ids)
                .push(")");
        }
        if let Some(ref scene_ids) = req.scene_ids
            && !scene_ids.is_empty()
        {
            self.push(" AND scene_ids @> ").push_bind(Json(scene_ids));
        }
        if let Some(ref mistake_tip_ids) = req.mistake_tip_ids
            && !mistake_tip_ids.is_empty()
        {
            self.push(" AND mistake_tip_ids @> ")
                .push_bind(Json(mistake_tip_ids));
        }
        self
    }
}

impl Question {
    // 添加题目-根据主键判断是新增还是更新
    pub async fn simple_save(pool: &PgPool, req: CreateQuestionReq) -> Result<i64, sqlx::Error> {
        let id: i64 = sqlx::query_scalar(
            r"
        INSERT INTO question (
            id, question_cate_id, question_type_id, question_tag_ids, author_id,
            source, original_name, status,
            title, content_plain, comment, difficulty_level,
            images, options, options_layout,
            answer, knowledge, analysis, process, remark, remark_ext,
            steps, question_dimension_ids, relation_type,
            level_id, scene_ids, mistake_tip_ids
        )
        VALUES (
            COALESCE($1, nextval('question_id_seq')), $2, $3, $4, $5,
            $6, $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
            $25, $26, $27
        )
        ON CONFLICT (id) DO UPDATE SET
            question_cate_id = EXCLUDED.question_cate_id,
            question_type_id = EXCLUDED.question_type_id,
            question_tag_ids = EXCLUDED.question_tag_ids,
            author_id = EXCLUDED.author_id,
            source = EXCLUDED.source,
            original_name = EXCLUDED.original_name,
            status = EXCLUDED.status,
            title = EXCLUDED.title,
            content_plain = EXCLUDED.content_plain,
            comment = EXCLUDED.comment,
            difficulty_level = EXCLUDED.difficulty_level,
            images = EXCLUDED.images,
            options = EXCLUDED.options,
            options_layout = EXCLUDED.options_layout,
            answer = EXCLUDED.answer,
            knowledge = EXCLUDED.knowledge,
            analysis = EXCLUDED.analysis,
            process = EXCLUDED.process,
            remark = EXCLUDED.remark,
            remark_ext = EXCLUDED.remark_ext,
            steps = EXCLUDED.steps,
            question_dimension_ids = EXCLUDED.question_dimension_ids,
            relation_type = EXCLUDED.relation_type,
            level_id = EXCLUDED.level_id,
            scene_ids = EXCLUDED.scene_ids,
            mistake_tip_ids = EXCLUDED.mistake_tip_ids,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        ",
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
        .bind(req.relation_type)
        .bind(req.level_id)
        .bind(Json(req.scene_ids))
        .bind(Json(req.mistake_tip_ids))
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
            r"
        INSERT INTO question (
            question_cate_id, question_type_id, question_tag_ids, author_id,
            source, original_name, status,
            title, content_plain, comment, difficulty_level,
            images, options, options_layout,
            answer, knowledge, analysis, process, remark, remark_ext,
            steps, question_dimension_ids, relation_type,
            level_id, scene_ids, mistake_tip_ids
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
        RETURNING *
        ",
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
        .bind(req.relation_type)
        .bind(req.level_id)
        .bind(Json(req.scene_ids))
        .bind(Json(req.mistake_tip_ids))
        .fetch_one(&mut **tx)
        .await
    }

    // tx 方式批量添加题题目
    // 注意事务签名类型 tx: &mut Transaction<'_, Postgres>,
    pub async fn tx_batch_insert(
        tx: &mut Transaction<'_, Postgres>,
        records: Vec<CreateQuestionReq>,
    ) -> Result<Vec<i64>, sqlx::Error> {
        // 请求参数为空则返回空即可
        if records.is_empty() {
            return Ok(vec![]);
        }

        // 预分配容量
        let mut all_ids = Vec::with_capacity(records.len());

        // 避免 SQL 语句过大
        // 简单看了下一个中等规模的题直接存为 .md 是 1.6k 500*1.6=800k, 大部分题都是选择填空一次性写300条应该暂时没什么风险
        for chunk in records.chunks(300) {
            let mut query_builder = QueryBuilder::new(
                r"
            INSERT INTO question (
                question_cate_id, question_type_id, question_tag_ids, author_id,source,original_name,
                title, content_plain, comment, difficulty_level,
                images, options, options_layout,
                answer, knowledge, analysis, process, remark,remark_ext,
                steps, question_dimension_ids, relation_type,
                level_id, scene_ids, mistake_tip_ids
            )
            ",
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
                    .push_bind(Json(req.question_dimension_ids.clone().unwrap_or_default()))
                    .push_bind(req.relation_type)
                    .push_bind(req.level_id)
                    .push_bind(Json(req.scene_ids.clone().unwrap_or_default()))
                    .push_bind(Json(req.mistake_tip_ids.clone().unwrap_or_default()));
            });

            query_builder.push(" RETURNING id");

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

    // 通过ids获取详情列表
    pub async fn find_by_ids(pool: &PgPool, ids: Vec<i64>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM question WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await
    }

    // 题型下题目数量
    pub async fn count_by_cate_and_type(
        pool: &PgPool,
        req: &CateAndTypeReq,
    ) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM question ");

        qb = qb.push_cate_and_type_where(req);

        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }

    // 题型下题目列表, 该接口没有扩展信息
    pub async fn list_by_cate_and_type(
        pool: &PgPool,
        req: &CateAndTypeReq,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let sql = r"
        SELECT
            id,
            question_cate_id,
            question_type_id,
            question_tag_ids,
            question_dimension_ids,
            relation_type,
            level_id,
            scene_ids,
            mistake_tip_ids,
            author_id,
            source,
            original_name,
            title,
            content_plain,
            comment,
            difficulty_level,
            images,
            options,
            status,
            options_layout,
            steps,
            approve_id,
            reject_reason,
            approve_at,
            created_at,
            updated_at
        FROM question
        ";
        let mut qb = QueryBuilder::new(sql);

        qb = qb.push_cate_and_type_where(req);

        qb.push(" ORDER BY id DESC LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }

    pub async fn exists_by_ext_id(pool: &PgPool, req: &ExtIdReq) -> Result<bool, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT EXISTS (SELECT 1 FROM question WHERE ");

        if let Some(type_id) = req.type_id
            && type_id > 0
        {
            qb.push("question_type_id = ").push_bind(type_id);
        } else if let Some(ref tag_ids) = req.tag_ids
            && !tag_ids.is_empty()
        {
            qb.push("question_tag_ids @> ").push_bind(Json(tag_ids));
        } else if let Some(ref dimension_ids) = req.dimension_ids
            && !dimension_ids.is_empty()
        {
            qb.push("question_dimension_ids @> ")
                .push_bind(Json(dimension_ids));
        } else if let Some(level_id) = req.level_id
            && level_id > 0
        {
            qb.push("level_id = ").push_bind(level_id);
        } else if let Some(ref scene_ids) = req.scene_ids
            && !scene_ids.is_empty()
        {
            qb.push("scene_ids @> ").push_bind(Json(scene_ids));
        } else if let Some(ref mistake_tip_ids) = req.mistake_tip_ids
            && !mistake_tip_ids.is_empty()
        {
            qb.push("mistake_tip_ids @> ")
                .push_bind(Json(mistake_tip_ids));
        } else {
            return Ok(false);
        }

        qb.push(")");

        qb.build_query_scalar::<bool>().fetch_one(pool).await
    }

    // 题型下是否存在题目
    pub async fn exist_by_cate_id(pool: &PgPool, cate_id: i32) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar::<_, bool>(
            r"
        SELECT EXISTS(SELECT 1 FROM question WHERE question_cate_id = $1)
        ",
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
        let now = Utc::now();

        let result = sqlx::query(
            r"
        UPDATE question
        SET status = $2,
            approve_id = $3,
            reject_reason = $4,
            approve_at = $5,
            updated_at = $5
        WHERE id = $1
        ",
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
        req: &SimilarReq,
    ) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new(
            "SELECT COUNT(q.id) FROM question q INNER JOIN question_relation qs ON q.id = qs.child_id",
        );

        qb = qb.push_similar_where(req);

        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }

    // 母题下面变式题列表
    pub async fn list_similar_by_params(
        pool: &PgPool,
        req: &SimilarReq,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut qb = QueryBuilder::new(
            "SELECT q.* FROM question q INNER JOIN question_relation qs ON q.id = qs.child_id",
        );

        qb = qb.push_similar_where(req);

        qb.push(" ORDER BY qs.id ASC ");
        qb.push(" LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }

    pub async fn delete_by_id(pool: &PgPool, id: i64) -> Result<u64, sqlx::Error> {
        let row = sqlx::query("DELETE FROM question WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(row.rows_affected())
    }

    // 题型等条件下题目列表
    pub async fn list_by_ext(
        pool: &PgPool,
        type_id: i16,
        req: &GenPaperGenConfig,
        limit: i16,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT * FROM question ");
        qb.push(" WHERE question_type_id = ").push_bind(type_id);

        qb = qb.push_random_where(req);

        qb.push(" ORDER BY RANDOM() LIMIT ").push_bind(limit);

        qb.build_query_as::<Self>().fetch_all(pool).await
    }
}
