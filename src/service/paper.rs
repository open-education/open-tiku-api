use crate::api::req::paper::{
    CommonPaperReq, DeleteReq, GenPaperGenConfig, GenPaperPreviewReq, PaperGenGroupReq,
    PaperGenReq, PaperListReq, TopPaperGroupReq, TopPaperReq,
};
use crate::api::resp::paper::{
    CommonPaperGenQuestionResp, CommonPaperGroupResp, CommonPaperResp, GenPaperGroupResp,
    GenPaperQuestionResp, GenPaperResp, PaperListResp, TopPaperGroupResp, TopPaperQuestionResp,
    TopPaperResp,
};
use crate::app::conf::AppState;
use crate::enums::paper::{PaperPageSource, PaperStatus, PaperType};
use crate::middleware::user::UserInfo;
use crate::model::paper::Paper;
use crate::model::paper_gen_config::{DifficultyLevelInfo, PaperGenConfig, QuestionTypeInfo};
use crate::model::paper_gen_question::PaperGenQuestion;
use crate::model::paper_group::PaperGroup;
use crate::model::paper_question::PaperQuestion;
use crate::model::question::Question;
use crate::service::question;
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use crate::util::local::to_local_datetime;
use chrono::Utc;
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use tracing::{error, info};

// 添加精选试卷
// 编辑试卷才用的模式是 主表 paper 根据主键更新, 字表 paper_group paper_question 采用先删除后重新写入的方法
pub async fn top_add(
    app_state: &AppState,
    req: TopPaperReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let db = &app_state.db;
    let is_update = req.common.id.is_some();

    // 参数验证
    validate_paper_top_request(&req)?;

    // 只允许编辑自己的试卷
    if let Some(id) = req.common.id {
        validate_is_allow_edit(db, id, user_info.user_id).await?;
    }

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to top begin transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    // 统计总题目数（在构建 Paper 之前）
    let total_question_count = req.groups.iter().map(|g| g.questions.len() as i32).sum();

    // 构建并插入试卷主体（包含总题目数）
    let paper = build_paper_meta_from_request(&user_info, &req.common, total_question_count);
    let paper_id = Paper::save(&mut tx, &paper).await.map_err(|err| {
        error!("Failed to insert top paper: {}", err);
        AppError::db_error("试卷主体信息添加失败")
    })?;

    // 构建题型和题目
    let (paper_groups, paper_questions) = build_top_groups_and_questions(paper_id, &req.groups);

    // 如果是编辑则需要先删除题型分类和题目列表
    if is_update {
        delete_top_info(&mut tx, paper_id, "top_add").await?;
    }

    // 批量插入题型
    if !paper_groups.is_empty() {
        PaperGroup::batch_insert(&mut tx, &paper_groups)
            .await
            .map_err(|err| {
                error!("Failed to insert top paper groups: {}", err);
                AppError::db_error("试卷题型信息添加失败")
            })?;
    }

    // 批量插入题目
    if !paper_questions.is_empty() {
        PaperQuestion::batch_insert(&mut tx, &paper_questions)
            .await
            .map_err(|err| {
                error!("Failed to insert top paper questions: {}", err);
                AppError::db_error("试卷题目信息添加失败")
            })?;
    }

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to top commit transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    // 记录操作日志
    info!(
        "Paper top added successfully. ID: {}, Title: {}, Total Questions: {}",
        paper_id, req.common.title, total_question_count
    );

    Ok(paper_id)
}

// 参数验证函数
fn validate_paper_top_request(req: &TopPaperReq) -> Result<(), AppError> {
    validate_paper_meta_request(&req.common)?;

    if req.groups.is_empty() {
        return Err(AppError::param_error("试卷至少需要一个题型"));
    }

    for (idx, group) in req.groups.iter().enumerate() {
        if group.type_name.trim().is_empty() {
            return Err(AppError::param_error(
                format!("第{}个题型名称不能为空", idx + 1).as_str(),
            ));
        }
        if group.questions.is_empty() {
            return Err(AppError::param_error(
                format!("题型'{}'至少需要一道题目", group.type_name).as_str(),
            ));
        }

        // 验证题目
        for (q_idx, question) in group.questions.iter().enumerate() {
            if question.stem.trim().is_empty() {
                return Err(AppError::param_error(
                    format!(
                        "题型'{}'的第{}道题目题干不能为空",
                        group.type_name,
                        q_idx + 1
                    )
                    .as_str(),
                ));
            }
            // 验证分数
            if question.score < 0 {
                return Err(AppError::param_error(
                    format!(
                        "题型'{}'的第{}道题目分数不能为负数",
                        group.type_name,
                        q_idx + 1
                    )
                    .as_str(),
                ));
            }
        }
    }

    Ok(())
}

// 只允许编辑自己的试卷
async fn validate_is_allow_edit(db: &PgPool, id: i64, user_id: i64) -> Result<(), AppError> {
    let has_paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Add paper err: {}", err);
            AppError::db_error("查询试卷信息错误")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if has_paper.author_id != user_id {
        return Err(AppError::permission_denied("只允许编辑自己的试卷"));
    }

    Ok(())
}

// 验证试卷基础必填字段
fn validate_paper_meta_request(req: &CommonPaperReq) -> Result<(), AppError> {
    // 考点名称或者学段等不能为空
    if req.related_id <= 0 {
        return Err(AppError::param_error("考点名称/学段导航不能为空"));
    }
    if req.tag.is_empty() {
        return Err(AppError::param_error("标签不能为空"));
    }
    if req.year.is_empty() {
        return Err(AppError::param_error("年份不能为空"));
    }
    if req.title.trim().is_empty() {
        return Err(AppError::param_error("试卷标题不能为空"));
    }

    // 草稿中和待审核支持编辑
    if !match PaperStatus::from_i16(req.status) {
        PaperStatus::Draft | PaperStatus::Pending => true,
        _ => false,
    } {
        return Err(AppError::business_error("试卷状态不支持编辑"));
    }

    Ok(())
}

// 构建试卷对象（包含总题目数）
fn build_paper_meta_from_request(
    user_info: &UserInfo,
    req: &CommonPaperReq,
    total_question_count: i32,
) -> Paper {
    Paper {
        id: req.id,
        related_id: req.related_id,
        related_name: req.related_name.clone(),
        paper_type: req.paper_type,
        tag: req.tag.clone(),
        year: req.year.clone(),
        grade: req.grade.clone(),
        semester: req.semester.clone(),
        title: req.title.clone(),
        score: req.score,
        source: req.source.clone(),
        remark: req.remark.clone(),
        author_id: user_info.user_id,
        author_name: user_info.username.clone().unwrap_or_default(),
        count: total_question_count, // 设置总题目数
        remark_ext: None,
        status: PaperStatus::from_i16(req.status) as i16,
        approve_id: 0,
        reject_reason: None,
        approve_at: None,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

// 构建题型和题目
fn build_top_groups_and_questions(
    paper_id: i64,
    groups: &[TopPaperGroupReq],
) -> (Vec<PaperGroup>, Vec<PaperQuestion>) {
    let group_count = groups.len();
    let mut paper_groups = Vec::with_capacity(group_count);

    // 预计算题目总数以优化内存分配
    let total_questions: usize = groups.iter().map(|g| g.questions.len()).sum();
    let mut paper_questions = Vec::with_capacity(total_questions);

    for (group_idx, group) in groups.iter().enumerate() {
        // 生成 group_id: 使用更大的基数避免冲突
        let group_id = paper_id * 1000 + (group_idx + 1) as i64;

        paper_groups.push(PaperGroup {
            id: group_id,
            paper_id,
            gen_id: group.gen_id.clone(),
            type_name: group.type_name.clone(),
            sub_title: group.sub_title.clone(),
        });

        // 构建该题型下的所有题目
        for question in &group.questions {
            paper_questions.push(PaperQuestion {
                id: 0,
                paper_id,
                group_id,
                gen_id: question.gen_id.clone(),
                order_num: question.order_num,
                stem: question.stem.clone(),
                images: question.images.clone(),
                options: question.options.clone(),
                options_layout: question.options_layout,
                answer: question.answer.clone(),
                analysis: question.analysis.clone(),
                score: question.score,
            });
        }
    }

    (paper_groups, paper_questions)
}

// 删除精选试卷题型分类和题目列表
async fn delete_top_info(
    tx: &mut Transaction<'_, Postgres>,
    paper_id: i64,
    source: &str,
) -> Result<(), AppError> {
    let del_group_rows = PaperGroup::delete_by_paper_id(tx, paper_id)
        .await
        .map_err(|err| {
            error!("Failed {} to delete top paper group: {}", source, err);
            AppError::db_error("删除题型分类失败")
        })?;
    info!(
        "Deleted {} top paper group rows: {:?}",
        source, del_group_rows
    );

    let del_question_rows = PaperQuestion::delete_by_paper_id(tx, paper_id)
        .await
        .map_err(|err| {
            error!("Failed {} to delete top paper question: {}", source, err);
            AppError::db_error("删除题目列表失败")
        })?;
    info!(
        "Deleted {} top paper question rows: {:?}",
        source, del_question_rows
    );

    Ok(())
}

// 精选试卷-试卷详情
pub async fn top_info(app_state: &AppState, id: i64) -> Result<TopPaperResp, AppError> {
    let db = &app_state.db;

    // 查询试卷主体
    let paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select top paper id: {}, error: {}", id, err);
            AppError::db_error("试卷查询出错")
        })?
        .ok_or_else(|| {
            error!("Select top paper id: {} is empty", id);
            AppError::not_found("试卷不存在")
        })?;

    // 查询题型
    let paper_groups = PaperGroup::find_by_paper_id(db, paper.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!(
                "Select top paper group, paper_id: {}, error: {}",
                paper.id.unwrap_or_default(),
                err
            );
            AppError::db_error("查询试卷题型失败")
        })?;

    // 如果有题型，才查询题目
    let paper_questions = if paper_groups.is_empty() {
        Vec::new()
    } else {
        let group_ids: Vec<i64> = paper_groups.iter().map(|g| g.id).collect();
        PaperQuestion::find_by_group_ids(db, paper.id.unwrap_or_default(), &group_ids)
            .await
            .map_err(|err| {
                error!(
                    "Select top paper question paper_id: {}, error: {}",
                    paper.id.unwrap_or_default(),
                    err
                );
                AppError::db_error("查询试卷题目失败")
            })?
    };

    Ok(to_top_resp(paper, paper_groups, paper_questions))
}

// 组装试卷详情返回
fn to_top_resp(
    paper: Paper,
    paper_groups: Vec<PaperGroup>,
    paper_questions: Vec<PaperQuestion>,
) -> TopPaperResp {
    let mut resp = TopPaperResp {
        common: paper.into(),
        groups: Vec::new(),
    };

    // 构建题型和题目的映射关系
    let mut questions_map: HashMap<i64, Vec<TopPaperQuestionResp>> = HashMap::new();

    for question in paper_questions {
        let group_id = question.group_id;
        let question_resp = question.into();
        questions_map
            .entry(group_id)
            .or_default()
            .push(question_resp);
    }

    let mut groups = Vec::with_capacity(paper_groups.len());
    for group in paper_groups {
        let group_resp = TopPaperGroupResp {
            questions: questions_map.remove(&group.id).unwrap_or_default(),
            common: group.into(),
        };
        groups.push(group_resp);
    }

    resp.groups = groups;

    resp
}

// 列表查询
pub async fn list(
    app_state: &AppState,
    req: PaperListReq,
    user_info: Option<UserInfo>,
) -> Result<PaperListResp, AppError> {
    let db = &app_state.db;

    // 检查参数
    if req.related_id <= 0 {
        return Err(AppError::param_error("考点/学段分类不能为空"));
    }

    // 页面来源
    let req_source = PaperPageSource::from_str(&req.source)
        .ok_or_else(|| AppError::param_error("不清楚的查询来源"))?;

    // 我的试卷等时需要登录
    let (author_id, status) = if req_source == PaperPageSource::List {
        (None, PaperStatus::Published as i16)
    } else {
        let user_info = user_info.ok_or_else(|| AppError::permission_denied("需要登录方能访问"))?;
        let status = req.status.unwrap_or(PaperStatus::Published as i16);
        if req_source == PaperPageSource::MyPaper {
            (Some(user_info.user_id), status)
        } else {
            (None, status)
        }
    };

    // 构建过滤条件
    let (where_clause, param_count) = Paper::build_condition(&req, author_id);

    // 查询总数
    let total = Paper::count(db, &req, author_id, status, &where_clause)
        .await
        .map_err(|err| {
            error!("Select paper count err: {}", err);
            AppError::db_error("查询试卷总数失败")
        })?;

    // 3查询列表
    let offset = (req.page_no - 1) * req.page_size;
    if offset >= total as i32 {
        return Ok(PaperListResp {
            list: vec![],
            page_no: req.page_no,
            page_size: req.page_size,
            total,
        });
    }

    let papers = Paper::list(
        db,
        &req,
        author_id,
        status,
        &where_clause,
        param_count,
        offset,
    )
    .await
    .map_err(|err| {
        error!("Select paper list err: {}", err);
        AppError::db_error("查询试卷列表失败")
    })?;

    Ok(PaperListResp {
        list: papers.into_iter().map(Into::into).collect(),
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

// 最新试卷
pub async fn latest(
    app_state: &AppState,
    path: (i16, i64),
) -> Result<Vec<CommonPaperResp>, AppError> {
    let papers = Paper::get_latest_papers(&app_state.db, path.0, path.1)
        .await
        .map_err(|err| {
            error!("Select paper list err: {}", err);
            AppError::db_error("查询试卷列表失败")
        })?;

    let list: Vec<CommonPaperResp> = papers.into_iter().map(Into::into).collect();

    Ok(list)
}

// 预览详情, 暂时还没有存表
pub async fn preview(
    app_state: &AppState,
    req: GenPaperPreviewReq,
    user_info: UserInfo,
) -> Result<GenPaperResp, AppError> {
    // 题型和题量非空
    if req.conf.question_cate_ids.is_empty() {
        return Err(AppError::param_error("题型不能为空"));
    }
    if req.conf.question_types.is_empty() {
        return Err(AppError::param_error("题量配置不能为空"));
    }

    let db = &app_state.db;

    let question_cate_ids = req.conf.question_cate_ids;
    let tag_ids = req.conf.tag_ids;
    let dimension_ids = req.conf.dimension_ids;
    let level_range = req.conf.level_range;
    let question_types = req.conf.question_types.clone();

    let created_at = to_local_datetime(Some(Utc::now()));
    let status = PaperStatus::Draft as i16;

    let paper_id = 1;
    let mut groups: Vec<GenPaperGroupResp> = vec![];

    // todo 难度等级不知道怎么实现

    let user_name = user_info.username.unwrap_or_default();

    // 每个类型并发执行, 后续等功能调整不大时再调整为并发执行
    for (index, question_type) in req.conf.question_types.iter().enumerate() {
        if question_type.num == 0 {
            continue;
        }

        let group_id = (index + 1) as i64;

        let rows = Question::list_by_ext(
            db,
            question_cate_ids.clone(),
            question_type.id,
            tag_ids.clone(),
            dimension_ids.clone(),
            question_type.num,
        )
        .await
        .map_err(|err| {
            error!("Select question err: {}", err);
            AppError::db_error("查询题目失败")
        })?;

        // 批量获取作者名称
        let mut user_ids_set = HashSet::with_capacity(rows.len() * 2);

        for q in &rows {
            user_ids_set.insert(q.author_id);
            if q.approve_id > 0 {
                user_ids_set.insert(q.approve_id);
            }
        }

        let user_ids: Vec<i64> = user_ids_set.into_iter().collect();
        let user_map: HashMap<i64, String> = get_user_map(db, user_ids).await?;

        let mut questions: Vec<GenPaperQuestionResp> = vec![];
        for (index, row) in rows.into_iter().enumerate() {
            let author_name = user_map
                .get(&row.author_id)
                .cloned()
                .unwrap_or_else(|| "".to_string());
            let approve_name = user_map
                .get(&row.approve_id)
                .cloned()
                .unwrap_or_else(|| "".to_string());

            questions.push(GenPaperQuestionResp {
                common: CommonPaperGenQuestionResp {
                    id: row.id,
                    paper_id: 0,
                    group_id,
                    gen_id: row.id.to_string(),
                    order_num: (index + 1) as i16,
                    question_id: row.id,
                    score: question_type.score as i32,
                },
                info: question::to_info_resp(&row, author_name, approve_name),
            });
        }

        groups.push(GenPaperGroupResp {
            common: CommonPaperGroupResp {
                id: group_id,
                paper_id,
                gen_id: group_id.to_string(),
                type_name: question_type.label.clone(),
                sub_title: Some(format!(
                    "本大题共{}个小题，每小题{}分，共{}分",
                    question_type.num,
                    question_type.score,
                    question_type.num * question_type.score
                )),
            },
            questions,
        });
    }

    Ok(GenPaperResp {
        common: CommonPaperResp {
            id: Some(paper_id),
            related_id: req.common.related_id,
            related_name: req.common.related_name,
            paper_type: req.common.paper_type,
            tag: req.common.tag,
            year: req.common.year,
            grade: req.common.grade,
            semester: req.common.semester,
            title: req.common.title,
            score: 0,
            source: req.common.source,
            author_id: user_info.user_id,
            author_name: user_name,
            status,
            status_desc: PaperStatus::desc(status).to_string(),
            approve_id: 0,
            reject_reason: None,
            approve_at: None,
            remark: req.common.remark,
            count: 0,
            created_at: created_at.clone(),
            updated_at: created_at,
        },
        conf: GenPaperGenConfig {
            question_cate_ids,
            tag_ids,
            dimension_ids,
            level_range,
            question_types,
        },
        groups,
    })
}

// 保存手动生成的试卷
pub async fn gen_add(
    app_state: &AppState,
    req: PaperGenReq,
    user_info: UserInfo,
) -> Result<i64, AppError> {
    let db = &app_state.db;
    let is_update = req.common.id.is_some();

    validate_paper_gen_request(&req)?;

    // 只允许编辑自己的试卷
    if let Some(id) = req.common.id {
        validate_is_allow_edit(db, id, user_info.user_id).await?;
    }

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to gen begin transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    // 构建并插入试卷主体（包含总题目数）
    let paper = build_paper_meta_from_request(
        &user_info,
        &req.common,
        req.common.count.unwrap_or_default(),
    );
    let paper_id = Paper::save(&mut tx, &paper).await.map_err(|err| {
        error!("Failed to insert gen paper: {}", err);
        AppError::db_error("试卷主体信息添加失败")
    })?;

    // 如果是更新则删除字表
    if is_update {
        delete_gen_info(&mut tx, paper_id, "gen_add").await?;
    }

    // 题目选择配置信息
    let paper_gen_config = build_gen_config_from_request(paper_id, &req.conf);
    let _ = PaperGenConfig::tx_insert(&mut tx, &paper_gen_config)
        .await
        .map_err(|err| {
            error!("Failed to insert gen paper gen config: {}", err);
            AppError::db_error("试卷题目选择配置信息添加失败")
        })?;

    // 构建题型和题目
    let (paper_groups, paper_questions) = build_gen_groups_and_questions(paper_id, &req.groups);

    // 批量插入题型
    if !paper_groups.is_empty() {
        PaperGroup::batch_insert(&mut tx, &paper_groups)
            .await
            .map_err(|err| {
                error!("Failed to insert gen paper groups: {}", err);
                AppError::db_error("试卷题型信息添加失败")
            })?;
    }

    // 批量插入题目
    if !paper_questions.is_empty() {
        PaperGenQuestion::batch_insert(&mut tx, &paper_questions)
            .await
            .map_err(|err| {
                error!("Failed to insert gen paper questions: {}", err);
                AppError::db_error("试卷题目信息添加失败")
            })?;
    }

    // 记录操作日志
    info!(
        "Paper gen added successfully. ID: {}, Title: {}, Total Questions: {}",
        paper_id,
        req.common.title,
        req.common.count.unwrap_or_default()
    );

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to gen commit transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    Ok(paper_id)
}

// 参数验证函数
fn validate_paper_gen_request(req: &PaperGenReq) -> Result<(), AppError> {
    validate_paper_meta_request(&req.common)?;

    if req.groups.is_empty() {
        return Err(AppError::param_error("试卷至少需要一个题型"));
    }

    for (idx, group) in req.groups.iter().enumerate() {
        if group.type_name.trim().is_empty() {
            return Err(AppError::param_error(
                format!("第{}个题型名称不能为空", idx + 1).as_str(),
            ));
        }
        if group.questions.is_empty() {
            return Err(AppError::param_error(
                format!("题型'{}'至少需要一道题目", group.type_name).as_str(),
            ));
        }

        // 验证题目
        for (q_idx, question) in group.questions.iter().enumerate() {
            // 验证分数
            if question.score < 0 {
                return Err(AppError::param_error(
                    format!(
                        "题型'{}'的第{}道题目分数不能为负数",
                        group.type_name,
                        q_idx + 1
                    )
                    .as_str(),
                ));
            }
        }
    }

    Ok(())
}

// 构建配置信息
fn build_gen_config_from_request(paper_id: i64, req: &GenPaperGenConfig) -> PaperGenConfig {
    let mut question_types: Vec<QuestionTypeInfo> = vec![];
    for info in req.question_types.iter() {
        question_types.push(QuestionTypeInfo {
            id: info.id,
            label: info.label.clone(),
            num: info.num,
            score: info.score,
        });
    }

    PaperGenConfig {
        paper_id,
        question_cate_ids: Json(req.question_cate_ids.clone()),
        question_tag_ids: Some(Json(req.tag_ids.clone().unwrap_or_default())),
        question_dimension_ids: Some(Json(req.dimension_ids.clone().unwrap_or_default())),
        question_type_info: Json(question_types),
        difficulty_level_info: Json(DifficultyLevelInfo {
            basic: req.level_range.basic,
            improve: req.level_range.improve,
            expand: req.level_range.expand,
        }),
    }
}

// 构建手动组卷题型和题目
fn build_gen_groups_and_questions(
    paper_id: i64,
    groups: &[PaperGenGroupReq],
) -> (Vec<PaperGroup>, Vec<PaperGenQuestion>) {
    let group_count = groups.len();
    let mut paper_groups = Vec::with_capacity(group_count);

    // 预计算题目总数以优化内存分配
    let total_questions: usize = groups.iter().map(|g| g.questions.len()).sum();
    let mut paper_questions = Vec::with_capacity(total_questions);

    for (group_idx, group) in groups.iter().enumerate() {
        // 生成 group_id: 使用更大的基数避免冲突
        let group_id = paper_id * 1000 + (group_idx + 1) as i64;

        paper_groups.push(PaperGroup {
            id: group_id,
            paper_id,
            gen_id: group.gen_id.clone(),
            type_name: group.type_name.clone(),
            sub_title: group.sub_title.clone(),
        });

        // 构建该题型下的所有题目
        for question in &group.questions {
            paper_questions.push(PaperGenQuestion {
                id: 0,
                paper_id,
                group_id,
                gen_id: question.gen_id.clone(),
                order_num: question.order_num,
                question_id: question.question_id,
                score: question.score,
            });
        }
    }

    (paper_groups, paper_questions)
}

// 删除试卷其它明细信息
async fn delete_gen_info(
    tx: &mut Transaction<'_, Postgres>,
    paper_id: i64,
    source: &str,
) -> Result<(), AppError> {
    let del_config_rows = PaperGenConfig::delete_by_paper_id(tx, paper_id)
        .await
        .map_err(|err| {
            error!("Failed {} to delete gen paper gen config: {}", source, err);
            AppError::db_error("删除试卷题型配置失败")
        })?;
    info!(
        "Deleted {} gen paper gen config rows: {:?}",
        source, del_config_rows
    );

    let del_group_rows = PaperGroup::delete_by_paper_id(tx, paper_id)
        .await
        .map_err(|err| {
            error!("Failed {} to delete gen paper group: {}", source, err);
            AppError::db_error("删除试卷题型分类失败")
        })?;
    info!(
        "Deleted {} gen paper group rows: {:?}",
        source, del_group_rows
    );

    let del_question_rows = PaperGenQuestion::delete_by_paper_id(tx, paper_id)
        .await
        .map_err(|err| {
            error!(
                "Failed {} to delete gen paper gen question: {}",
                source, err
            );
            AppError::db_error("删除试卷题目列表失败")
        })?;

    info!(
        "Deleted {} gen paper gen question rows: {:?}",
        source, del_question_rows
    );

    Ok(())
}

// 手动组卷-试卷详情
pub async fn gen_info(app_state: &AppState, id: i64) -> Result<GenPaperResp, AppError> {
    let db = &app_state.db;

    // 查询试卷主体
    let paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select gen paper id: {}, error: {}", id, err);
            AppError::db_error("试卷信息查询错误")
        })?
        .ok_or_else(|| {
            error!("Select gen paper id: {} is empty", id);
            AppError::not_found("试卷不存在")
        })?;

    // 配置信息
    let gen_conf = PaperGenConfig::find_by_paper_id(db, id)
        .await
        .map_err(|err| {
            error!("Select gen paper gen config id: {}, error: {}", id, err);
            AppError::db_error("试卷配置信息查询错误")
        })?
        .ok_or_else(|| {
            error!("Select gen paper gen config id: {} is empty", id);
            AppError::not_found("试卷配置信息不存在")
        })?;

    // 查询题型
    let paper_groups = PaperGroup::find_by_paper_id(db, paper.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!(
                "Select gen paper group, paper_id: {}, error: {}",
                paper.id.unwrap_or_default(),
                err
            );
            AppError::db_error("查询试卷题型失败")
        })?;

    // 如果有题型，才查询题目
    let paper_gen_questions = if paper_groups.is_empty() {
        Vec::new()
    } else {
        let group_ids: Vec<i64> = paper_groups.iter().map(|g| g.id).collect();
        PaperGenQuestion::find_by_group_ids(db, paper.id.unwrap_or_default(), &group_ids)
            .await
            .map_err(|err| {
                error!(
                    "Select gen paper gen question paper_id: {}, error: {}",
                    paper.id.unwrap_or_default(),
                    err
                );
                AppError::db_error("查询试卷题目失败")
            })?
    };

    // 获取真正的题目信息
    let question_ids: Vec<i64> = paper_gen_questions.iter().map(|q| q.question_id).collect();
    let questions = Question::find_by_ids(db, question_ids)
        .await
        .map_err(|err| {
            error!(
                "Select gen paper gen question info paper_id: {}, error: {}",
                paper.id.unwrap_or_default(),
                err
            );
            AppError::db_error("查询试卷题目详情失败")
        })?;

    // 收集题型标识和作者信息
    let (question_map, author_ids) =
        questions
            .into_iter()
            .fold((HashMap::new(), Vec::new()), |(mut map, mut ids), q| {
                ids.push(q.author_id);
                if q.approve_id > 0 {
                    ids.push(q.approve_id);
                }
                map.insert(q.id, q);
                (map, ids)
            });
    let user_map: HashMap<i64, String> = get_user_map(db, author_ids).await?;

    to_gen_resp(
        paper,
        gen_conf,
        paper_groups,
        paper_gen_questions,
        &user_map,
        question_map,
    )
}

// 手动组卷-试卷详情返回
fn to_gen_resp(
    paper: Paper,
    paper_gen_config: PaperGenConfig,
    paper_groups: Vec<PaperGroup>,
    paper_questions: Vec<PaperGenQuestion>,
    user_map: &HashMap<i64, String>,
    question_raw_map: HashMap<i64, Question>,
) -> Result<GenPaperResp, AppError> {
    let mut resp = GenPaperResp {
        common: paper.into(),
        conf: GenPaperGenConfig {
            question_cate_ids: paper_gen_config.question_cate_ids.0,
            tag_ids: paper_gen_config.question_tag_ids.map(|j| j.0),
            dimension_ids: paper_gen_config.question_dimension_ids.map(|j| j.0),
            level_range: paper_gen_config.difficulty_level_info.0,
            question_types: paper_gen_config.question_type_info.0,
        },
        groups: vec![],
    };

    // 构建题型和题目的映射关系
    let mut questions_map: HashMap<i64, Vec<GenPaperQuestionResp>> = HashMap::new();

    for question in paper_questions {
        let group_id = question.group_id;
        let raw = question_raw_map.get(&question.question_id).ok_or_else(|| {
            error!(
                "gen group_id {} question_id {} not found in map",
                group_id, question.question_id
            );
            AppError::param_error("题目不存在")
        })?;

        let question_resp = to_gen_paper_question_resp(question, raw, user_map);
        questions_map
            .entry(group_id)
            .or_default()
            .push(question_resp);
    }

    let mut groups = Vec::with_capacity(paper_groups.len());
    for group in paper_groups {
        let group_resp = GenPaperGroupResp {
            questions: questions_map.remove(&group.id).unwrap_or_default(),
            common: group.into(),
        };
        groups.push(group_resp);
    }

    resp.groups = groups;

    Ok(resp)
}

fn to_gen_paper_question_resp(
    gen_info: PaperGenQuestion,
    row: &Question,
    user_map: &HashMap<i64, String>,
) -> GenPaperQuestionResp {
    GenPaperQuestionResp {
        common: gen_info.into(),

        info: question::to_info_resp(
            row,
            user_map
                .get(&row.author_id)
                .cloned()
                .unwrap_or_else(|| "".to_string()),
            user_map
                .get(&row.approve_id)
                .cloned()
                .unwrap_or_else(|| "".to_string()),
        ),
    }
}

// 删除试卷
pub async fn delete(
    app_state: &AppState,
    req: DeleteReq,
    user_info: UserInfo,
) -> Result<bool, AppError> {
    if req.id <= 0 {
        return Err(AppError::param_error("试卷标识为空"));
    }

    let db = &app_state.db;

    // 只允许删除自己的试卷
    let has_paper = Paper::find_by_id(db, req.id)
        .await
        .map_err(|err| {
            error!("Failed to find paper: {}", err);
            AppError::db_error("试卷查询错误")
        })?
        .ok_or_else(|| AppError::not_found("试卷不存在"))?;
    if has_paper.author_id != user_info.user_id {
        return Err(AppError::permission_denied("只允许删除自己的试卷"));
    }

    // 只有草稿的试卷可以删除
    if has_paper.status != PaperStatus::Draft as i16 {
        return Err(AppError::business_error("只允许删除草稿中的试卷"));
    }

    let rows = Paper::delete(db, req.id).await.map_err(|err| {
        error!("paper delete by id err: {:?}", err);
        AppError::db_error("删除失败")
    })?;

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed delete to gen begin transaction: {}", e);
        AppError::db_error("启动事务失败")
    })?;

    // 删除试卷明细
    match has_paper.paper_type {
        t if t == PaperType::Top as i16 => {
            delete_top_info(&mut tx, req.id, "delete").await?;
        }
        t if t == PaperType::Gen as i16 => {
            delete_gen_info(&mut tx, req.id, "delete").await?;
        }
        _ => {
            error!(
                "Failed delete paper info: {}, unknown paper_type: {}",
                req.id, has_paper.paper_type
            );
        }
    }

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed delete to gen commit transaction: {}", e);
        AppError::db_error("提交事务失败")
    })?;

    Ok(rows > 0)
}
