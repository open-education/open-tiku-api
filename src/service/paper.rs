use crate::AppConfig;
use crate::api::paper::{
    CommonPaperGenQuestionResp, CommonPaperGroupResp, CommonPaperReq, CommonPaperResp,
    GenPaperConfigReq, GenPaperGroupResp, GenPaperPreviewReq, GenPaperQuestionResp, GenPaperResp,
    PaperGenGroupReq, PaperGenReq, PaperListReq, PaperListResp, TopPaperGroupReq,
    TopPaperGroupResp, TopPaperQuestionResp, TopPaperReq, TopPaperResp,
};
use crate::middleware::user::UserInfo;
use crate::model::paper::{Paper, PaperStatus};
use crate::model::paper_gen_config::{DifficultyLevelInfo, PaperGenConfig, QuestionTypeInfo};
use crate::model::paper_gen_question::PaperGenQuestion;
use crate::model::paper_group::PaperGroup;
use crate::model::paper_question::PaperQuestion;
use crate::model::question::Question;
use crate::service::question::to_info_resp;
use crate::util::local::to_local_datetime;
use actix_web::web;
use chrono::Utc;
use log::{error, info};
use sqlx::PgPool;
use sqlx::types::Json;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

// 添加精选试卷
// 编辑试卷才用的模式是 主表 paper 根据主键更新, 字表 paper_group paper_question 采用先删除后重新写入的方法
pub async fn top_add(
    app_conf: web::Data<AppConfig>,
    req: TopPaperReq,
    user_info: UserInfo,
) -> Result<i64, Error> {
    let db = &app_conf.db;
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
        Error::new(ErrorKind::Other, "启动事务失败")
    })?;

    // 统计总题目数（在构建 Paper 之前）
    let total_question_count = req.groups.iter().map(|g| g.questions.len() as i32).sum();

    // 构建并插入试卷主体（包含总题目数）
    let paper = build_paper_meta_from_request(&user_info, &req.common, total_question_count);
    let paper_id = Paper::save(&mut tx, &paper).await.map_err(|err| {
        error!("Failed to insert top paper: {}", err);
        Error::new(ErrorKind::Other, "试卷主体信息添加失败")
    })?;

    // 构建题型和题目
    let (paper_groups, paper_questions) = build_top_groups_and_questions(paper_id, &req.groups);

    // 如果是编辑则需要先删除题型分类和题目列表
    if is_update {
        let del_group_rows =
            PaperGroup::delete_by_paper_id(&mut tx, req.common.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete top paper group: {}", err);
                    Error::new(ErrorKind::Other, "删除题型分类失败")
                })?;
        info!("Deleted top paper group rows: {:?}", del_group_rows);

        let del_question_rows =
            PaperQuestion::delete_by_paper_id(&mut tx, req.common.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete top paper question: {}", err);
                    Error::new(ErrorKind::Other, "删除题目列表失败")
                })?;
        info!("Deleted top paper question rows: {:?}", del_question_rows);
    }

    // 批量插入题型
    if !paper_groups.is_empty() {
        PaperGroup::batch_insert(&mut tx, &paper_groups)
            .await
            .map_err(|err| {
                error!("Failed to insert top paper groups: {}", err);
                Error::new(ErrorKind::Other, "试卷题型信息添加失败")
            })?;
    }

    // 批量插入题目
    if !paper_questions.is_empty() {
        PaperQuestion::batch_insert(&mut tx, &paper_questions)
            .await
            .map_err(|err| {
                error!("Failed to insert top paper questions: {}", err);
                Error::new(ErrorKind::Other, "试卷题目信息添加失败")
            })?;
    }

    // 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to top commit transaction: {}", e);
        Error::new(ErrorKind::Other, "提交事务失败")
    })?;

    // 记录操作日志
    info!(
        "Paper top added successfully. ID: {}, Title: {}, Total Questions: {}",
        paper_id, req.common.title, total_question_count
    );

    Ok(paper_id)
}

// 参数验证函数
fn validate_paper_top_request(req: &TopPaperReq) -> Result<(), Error> {
    validate_paper_meta_request(&req.common)?;

    if req.groups.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "试卷至少需要一个题型"));
    }

    for (idx, group) in req.groups.iter().enumerate() {
        if group.type_name.trim().is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("第{}个题型名称不能为空", idx + 1),
            ));
        }
        if group.questions.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("题型'{}'至少需要一道题目", group.type_name),
            ));
        }

        // 验证题目
        for (q_idx, question) in group.questions.iter().enumerate() {
            if question.stem.trim().is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "题型'{}'的第{}道题目题干不能为空",
                        group.type_name,
                        q_idx + 1
                    ),
                ));
            }
            // 验证分数
            if question.score < 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "题型'{}'的第{}道题目分数不能为负数",
                        group.type_name,
                        q_idx + 1
                    ),
                ));
            }
        }
    }

    Ok(())
}

// 只允许编辑自己的试卷
async fn validate_is_allow_edit(db: &PgPool, id: i64, user_id: i64) -> Result<(), Error> {
    let has_paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Add paper err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷信息错误")
        })?
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "试卷不存在"))?;
    if has_paper.author_id != user_id {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "只允许编辑自己的试卷",
        ));
    }

    Ok(())
}

// 验证试卷基础必填字段
fn validate_paper_meta_request(req: &CommonPaperReq) -> Result<(), Error> {
    // 考点名称或者学段等不能为空
    if req.related_id <= 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "考点名称/学段导航不能为空",
        ));
    }
    if req.tag.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "标签不能为空"));
    }
    if req.year.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "年份不能为空"));
    }
    if req.title.trim().is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "试卷标题不能为空"));
    }

    // 草稿中和待审核支持编辑
    let paper_status = PaperStatus::from_i16(req.status).as_i16();
    if !vec![PaperStatus::Draft.as_i16(), PaperStatus::Pending.as_i16()].contains(&paper_status) {
        return Err(Error::new(ErrorKind::InvalidInput, "试卷状态不支持编辑"));
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
        status: PaperStatus::from_i16(req.status).as_i16(),
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

// 精选试卷-试卷详情
pub async fn top_info(app_conf: web::Data<AppConfig>, id: i64) -> Result<TopPaperResp, Error> {
    let db = &app_conf.db;

    // 查询试卷主体
    let paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select top paper id: {}, error: {}", id, err);
            Error::new(ErrorKind::NotFound, "试卷不存在")
        })?
        .ok_or_else(|| {
            error!("Select top paper id: {} is empty", id);
            Error::new(ErrorKind::NotFound, "试卷不存在")
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
            Error::new(ErrorKind::Other, "查询试卷题型失败")
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
                Error::new(ErrorKind::Other, "查询试卷题目失败")
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
        common: to_common_paper_resp(paper),
        groups: Vec::new(),
    };

    // 构建题型和题目的映射关系
    let mut questions_map: HashMap<i64, Vec<TopPaperQuestionResp>> = HashMap::new();

    for question in paper_questions {
        let group_id = question.group_id;
        let question_resp = to_top_paper_question_resp(question);
        questions_map
            .entry(group_id)
            .or_insert_with(Vec::new)
            .push(question_resp);
    }

    let mut groups = Vec::with_capacity(paper_groups.len());
    for group in paper_groups {
        let group_resp = TopPaperGroupResp {
            common: to_common_paper_group_resp(&group),
            questions: questions_map.remove(&group.id).unwrap_or_default(),
        };
        groups.push(group_resp);
    }

    resp.groups = groups;

    resp
}

fn to_common_paper_resp(row: Paper) -> CommonPaperResp {
    CommonPaperResp {
        id: row.id,
        related_id: row.related_id,
        related_name: row.related_name,
        paper_type: row.paper_type,
        tag: row.tag,
        year: row.year,
        grade: row.grade,
        semester: row.semester,
        title: row.title,
        score: row.score,
        source: row.source,
        author_id: row.author_id,
        author_name: row.author_name,
        status: row.status,
        status_desc: PaperStatus::desc(row.status),
        approve_id: row.approve_id,
        reject_reason: row.reject_reason,
        approve_at: None,
        remark: row.remark,
        count: row.count,
        created_at: to_local_datetime(row.created_at),
        updated_at: to_local_datetime(row.updated_at),
    }
}

fn to_common_paper_group_resp(row: &PaperGroup) -> CommonPaperGroupResp {
    CommonPaperGroupResp {
        id: row.id,
        paper_id: row.paper_id,
        gen_id: row.gen_id.clone(),
        type_name: row.type_name.clone(),
        sub_title: row.sub_title.clone(),
    }
}

// 转换为 PaperQuestionResp
fn to_top_paper_question_resp(row: PaperQuestion) -> TopPaperQuestionResp {
    TopPaperQuestionResp {
        id: row.id,
        paper_id: row.paper_id,
        group_id: row.group_id,
        gen_id: row.gen_id,
        order_num: row.order_num,
        stem: row.stem,
        images: row.images,
        options: row.options,
        options_layout: row.options_layout,
        answer: row.answer,
        analysis: row.analysis,
        score: row.score,
    }
}

// 列表查询
pub async fn list(
    app_conf: web::Data<AppConfig>,
    req: PaperListReq,
    user_info: Option<UserInfo>,
) -> Result<PaperListResp, Error> {
    let db = &app_conf.db;

    // 检查参数
    if req.related_id <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "考点/学段分类不能为空"));
    }

    // 我的试卷等时需要登录
    let (author_id, status) = if req.source == "list" {
        (None, PaperStatus::Published as i16)
    } else {
        let user_info =
            user_info.ok_or_else(|| Error::new(ErrorKind::PermissionDenied, "需要登录方能访问"))?;
        let status = req.status.unwrap_or(PaperStatus::Published as i16);
        (Some(user_info.user_id), status)
    };

    // 1. 构建过滤条件
    let (where_clause, param_count) = Paper::build_condition(&req, author_id);

    // 2. 查询总数
    let total = Paper::count(db, &req, author_id, status, &where_clause)
        .await
        .map_err(|err| {
            error!("Select paper count err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷总数失败")
        })?;

    // 3. 查询列表
    let papers = Paper::list(db, &req, author_id, status, &where_clause, param_count)
        .await
        .map_err(|err| {
            error!("Select paper list err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷列表失败")
        })?;

    let list: Vec<CommonPaperResp> = papers.into_iter().map(to_common_paper_resp).collect();

    Ok(PaperListResp {
        list,
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

// 最新试卷
pub async fn latest(
    app_conf: web::Data<AppConfig>,
    count: i64,
) -> Result<Vec<CommonPaperResp>, Error> {
    let papers = Paper::get_latest_papers(&app_conf.db, count)
        .await
        .map_err(|err| {
            error!("Select paper list err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷列表失败")
        })?;

    let list: Vec<CommonPaperResp> = papers.into_iter().map(to_common_paper_resp).collect();

    Ok(list)
}

// 预览详情, 暂时还没有存表
pub async fn preview(
    app_conf: web::Data<AppConfig>,
    req: GenPaperPreviewReq,
    user_info: UserInfo,
) -> Result<GenPaperResp, Error> {
    // 题型和题量非空
    if req.conf.question_cate_ids.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "题型不能为空"));
    }
    if req.conf.question_types.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "题量配置不能为空"));
    }

    let db = &app_conf.db;

    let question_cate_ids = req.conf.question_cate_ids;
    let tag_ids = req.conf.tag_ids;
    let dimension_ids = req.conf.dimension_ids;

    let created_at = to_local_datetime(Utc::now());
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
            Error::new(ErrorKind::Other, "查询题目失败")
        })?;

        let mut questions: Vec<GenPaperQuestionResp> = vec![];
        for (index, row) in rows.into_iter().enumerate() {
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
                info: to_info_resp(&row, user_name.clone()),
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
            status_desc: PaperStatus::desc(status),
            approve_id: 0,
            reject_reason: None,
            approve_at: None,
            remark: req.common.remark,
            count: 0,
            created_at: created_at.clone(),
            updated_at: created_at,
        },
        groups,
    })
}

// 保存手动生成的试卷
pub async fn gen_add(
    app_conf: web::Data<AppConfig>,
    req: PaperGenReq,
    user_info: UserInfo,
) -> Result<i64, Error> {
    let db = &app_conf.db;
    let is_update = req.common.id.is_some();

    validate_paper_gen_request(&req)?;

    // 只允许编辑自己的试卷
    if let Some(id) = req.common.id {
        validate_is_allow_edit(db, id, user_info.user_id).await?;
    }

    // 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to gen begin transaction: {}", e);
        Error::new(ErrorKind::Other, "启动事务失败")
    })?;

    // 构建并插入试卷主体（包含总题目数）
    let paper = build_paper_meta_from_request(
        &user_info,
        &req.common,
        req.common.count.unwrap_or_default(),
    );
    let paper_id = Paper::save(&mut tx, &paper).await.map_err(|err| {
        error!("Failed to insert gen paper: {}", err);
        Error::new(ErrorKind::Other, "试卷主体信息添加失败")
    })?;

    // 如果是更新则删除字表
    if is_update {
        let del_config_rows =
            PaperGenConfig::delete_by_paper_id(&mut tx, req.common.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete gen paper gen config: {}", err);
                    Error::new(ErrorKind::Other, "删除题型配置失败")
                })?;
        info!("Deleted gen paper gen config rows: {:?}", del_config_rows);

        let del_group_rows =
            PaperGroup::delete_by_paper_id(&mut tx, req.common.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete gen paper group: {}", err);
                    Error::new(ErrorKind::Other, "删除题型分类失败")
                })?;
        info!("Deleted gen paper group rows: {:?}", del_group_rows);

        let del_question_rows =
            PaperGenQuestion::delete_by_paper_id(&mut tx, req.common.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete gen paper gen question: {}", err);
                    Error::new(ErrorKind::Other, "删除题目列表失败")
                })?;
        info!(
            "Deleted gen paper gen question rows: {:?}",
            del_question_rows
        );
    }

    // 题目选择配置信息
    let paper_gen_config = build_gen_config_from_request(paper_id, &req.conf);
    let _ = PaperGenConfig::tx_insert(&mut tx, &paper_gen_config)
        .await
        .map_err(|err| {
            error!("Failed to insert gen paper gen config: {}", err);
            Error::new(ErrorKind::Other, "试卷题目选择配置信息添加失败")
        })?;

    // 构建题型和题目
    let (paper_groups, paper_questions) = build_gen_groups_and_questions(paper_id, &req.groups);

    // 批量插入题型
    if !paper_groups.is_empty() {
        PaperGroup::batch_insert(&mut tx, &paper_groups)
            .await
            .map_err(|err| {
                error!("Failed to insert gen paper groups: {}", err);
                Error::new(ErrorKind::Other, "试卷题型信息添加失败")
            })?;
    }

    // 批量插入题目
    if !paper_questions.is_empty() {
        PaperGenQuestion::batch_insert(&mut tx, &paper_questions)
            .await
            .map_err(|err| {
                error!("Failed to insert gen paper questions: {}", err);
                Error::new(ErrorKind::Other, "试卷题目信息添加失败")
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
        Error::new(ErrorKind::Other, "提交事务失败")
    })?;

    Ok(paper_id)
}

// 参数验证函数
fn validate_paper_gen_request(req: &PaperGenReq) -> Result<(), Error> {
    validate_paper_meta_request(&req.common)?;

    if req.groups.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "试卷至少需要一个题型"));
    }

    for (idx, group) in req.groups.iter().enumerate() {
        if group.type_name.trim().is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("第{}个题型名称不能为空", idx + 1),
            ));
        }
        if group.questions.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("题型'{}'至少需要一道题目", group.type_name),
            ));
        }

        // 验证题目
        for (q_idx, question) in group.questions.iter().enumerate() {
            // 验证分数
            if question.score < 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "题型'{}'的第{}道题目分数不能为负数",
                        group.type_name,
                        q_idx + 1
                    ),
                ));
            }
        }
    }

    Ok(())
}

// 构建配置信息
fn build_gen_config_from_request(paper_id: i64, req: &GenPaperConfigReq) -> PaperGenConfig {
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
        id: 0,
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

// 手动组卷-试卷详情
pub async fn gen_info(app_conf: web::Data<AppConfig>, id: i64) -> Result<GenPaperResp, Error> {
    let db = &app_conf.db;

    // 查询试卷主体
    let paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select gen paper id: {}, error: {}", id, err);
            Error::new(ErrorKind::NotFound, "试卷不存在")
        })?
        .ok_or_else(|| {
            error!("Select gen paper id: {} is empty", id);
            Error::new(ErrorKind::NotFound, "试卷不存在")
        })?;

    // 配置信息

    // 查询题型
    let paper_groups = PaperGroup::find_by_paper_id(db, paper.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!(
                "Select gen paper group, paper_id: {}, error: {}",
                paper.id.unwrap_or_default(),
                err
            );
            Error::new(ErrorKind::Other, "查询试卷题型失败")
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
                Error::new(ErrorKind::Other, "查询试卷题目失败")
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
            Error::new(ErrorKind::Other, "查询试卷题目详情失败")
        })?;
    let question_map: HashMap<i64, Question> = questions.into_iter().map(|q| (q.id, q)).collect();

    to_gen_resp(paper, paper_groups, paper_gen_questions, question_map)
}

// 手动组卷-试卷详情返回
fn to_gen_resp(
    paper: Paper,
    paper_groups: Vec<PaperGroup>,
    paper_questions: Vec<PaperGenQuestion>,
    question_raw_map: HashMap<i64, Question>,
) -> Result<GenPaperResp, Error> {
    let mut resp = GenPaperResp {
        common: to_common_paper_resp(paper),
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
            Error::new(ErrorKind::Other, "题目不存在")
        })?;

        let question_resp = to_gen_paper_question_resp(question, raw);
        questions_map
            .entry(group_id)
            .or_insert_with(Vec::new)
            .push(question_resp);
    }

    let mut groups = Vec::with_capacity(paper_groups.len());
    for group in paper_groups {
        let group_resp = GenPaperGroupResp {
            common: to_common_paper_group_resp(&group),
            questions: questions_map.remove(&group.id).unwrap_or_default(),
        };
        groups.push(group_resp);
    }

    resp.groups = groups;

    Ok(resp)
}

fn to_gen_paper_question_resp(gen_info: PaperGenQuestion, row: &Question) -> GenPaperQuestionResp {
    GenPaperQuestionResp {
        common: CommonPaperGenQuestionResp {
            id: gen_info.id,
            paper_id: gen_info.paper_id,
            group_id: gen_info.group_id,
            gen_id: gen_info.gen_id,
            order_num: gen_info.order_num,
            question_id: gen_info.question_id,
            score: gen_info.score,
        },

        info: to_info_resp(row, "".to_string()),
    }
}
