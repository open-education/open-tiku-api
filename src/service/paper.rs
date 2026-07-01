use crate::AppConfig;
use crate::api::paper::{
    PaperGroupReq, PaperGroupResp, PaperListReq, PaperListResp, PaperQuestionResp, PaperReq,
    PaperResp,
};
use crate::constant::meta;
use crate::model::paper::{Paper, PaperStatus};
use crate::model::paper_group::PaperGroup;
use crate::model::paper_question::PaperQuestion;
use crate::util::local::to_local_datetime;
use actix_web::web;
use log::{error, info};
use std::io::{Error, ErrorKind};

// 添加试卷
// 编辑试卷才用的模式是 主表 paper 根据主键更新, 字表 paper_group paper_question 采用先删除后重新写入的方法
pub async fn add(app_conf: web::Data<AppConfig>, req: PaperReq) -> Result<i64, Error> {
    let db = &app_conf.db;
    let is_update = req.id.is_some();

    // 1. 参数验证
    validate_paper_request(&req)?;

    // 2. 开启事务
    let mut tx = db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {}", e);
        Error::new(ErrorKind::Other, "启动事务失败")
    })?;

    // 3. 统计总题目数（在构建 Paper 之前）
    let total_question_count = req.groups.iter().map(|g| g.questions.len() as i32).sum();

    // 4. 构建并插入试卷主体（包含总题目数）
    let paper = build_paper_from_request(&req, total_question_count);
    let paper_id = Paper::save(&mut tx, &paper).await.map_err(|err| {
        error!("Failed to insert paper: {}", err);
        Error::new(ErrorKind::Other, "试卷主体信息添加失败")
    })?;

    // 5. 构建题型和题目
    let (paper_groups, paper_questions) = build_groups_and_questions(paper_id, &req.groups);

    // 如果是编辑则需要先删除题型分类和题目列表
    if is_update {
        let del_group_rows = PaperGroup::delete_by_paper_id(&mut tx, req.id.unwrap_or_default())
            .await
            .map_err(|err| {
                error!("Failed to delete paper group: {}", err);
                Error::new(ErrorKind::Other, "删除题型分类失败")
            })?;
        info!("Deleted paper paper group rows: {:?}", del_group_rows);

        let del_question_rows =
            PaperQuestion::delete_by_paper_id(&mut tx, req.id.unwrap_or_default())
                .await
                .map_err(|err| {
                    error!("Failed to delete paper question: {}", err);
                    Error::new(ErrorKind::Other, "删除题目列表失败")
                })?;
        info!("Deleted paper paper question rows: {:?}", del_question_rows);
    }

    // 6. 批量插入题型
    if !paper_groups.is_empty() {
        PaperGroup::batch_insert(&mut tx, &paper_groups)
            .await
            .map_err(|err| {
                error!("Failed to insert paper groups: {}", err);
                Error::new(ErrorKind::Other, "试卷题型信息添加失败")
            })?;
    }

    // 7. 批量插入题目
    if !paper_questions.is_empty() {
        PaperQuestion::batch_insert(&mut tx, &paper_questions)
            .await
            .map_err(|err| {
                error!("Failed to insert paper questions: {}", err);
                Error::new(ErrorKind::Other, "试卷题目信息添加失败")
            })?;
    }

    // 8. 提交事务
    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {}", e);
        Error::new(ErrorKind::Other, "提交事务失败")
    })?;

    // 9. 记录操作日志
    info!(
        "Paper added successfully. ID: {}, Title: {}, Total Questions: {}",
        paper_id, req.title, total_question_count
    );

    Ok(paper_id)
}

// 参数验证函数（增强版）
fn validate_paper_request(req: &PaperReq) -> Result<(), Error> {
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
    if req.title.trim().is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "试卷标题不能为空"));
    }
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

// 构建试卷对象（包含总题目数）
fn build_paper_from_request(req: &PaperReq, total_question_count: i32) -> Paper {
    Paper {
        id: req.id,
        related_id: req.related_id,
        related_name: req.related_name.clone(),
        tag: req.tag.clone(),
        year: req.year.clone(),
        grade: req.grade.clone(),
        semester: req.semester.clone(),
        title: req.title.clone(),
        score: req.score,
        source: req.source.clone(),
        remark: req.remark.clone(),
        author_id: meta::TEMP_ADMIN_ID, // TODO: 从认证上下文获取
        author_name: "admin".to_string(),
        count: total_question_count, // 设置总题目数
        remark_ext: None,
        status: req.status,
        approve_id: 0,
        reject_reason: None,
        approve_at: None,
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

// 构建题型和题目（优化版本）
fn build_groups_and_questions(
    paper_id: i64,
    groups: &[PaperGroupReq],
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

// 试卷详情
pub async fn info(app_conf: web::Data<AppConfig>, id: i64) -> Result<PaperResp, Error> {
    let db = &app_conf.db;

    // 1. 查询试卷主体
    let paper = Paper::find_by_id(db, id)
        .await
        .map_err(|err| {
            error!("Select paper id: {}, error: {}", id, err);
            Error::new(ErrorKind::NotFound, "试卷不存在")
        })?
        .ok_or_else(|| {
            error!("Select paper id: {} is empty", id);
            Error::new(ErrorKind::NotFound, "试卷不存在")
        })?;

    // 2. 查询题型
    let paper_groups = PaperGroup::find_by_paper_id(db, paper.id.unwrap_or_default())
        .await
        .map_err(|err| {
            error!(
                "Select paper group, paper_id: {}, error: {}",
                paper.id.unwrap_or_default(),
                err
            );
            Error::new(ErrorKind::Other, "查询试卷题型失败")
        })?;

    // 3. 如果有题型，才查询题目
    let paper_questions = if paper_groups.is_empty() {
        Vec::new()
    } else {
        let group_ids: Vec<i64> = paper_groups.iter().map(|g| g.id).collect();
        PaperQuestion::find_by_group_ids(db, paper.id.unwrap_or_default(), &group_ids)
            .await
            .map_err(|err| {
                error!(
                    "Select paper question paper_id: {}, error: {}",
                    paper.id.unwrap_or_default(),
                    err
                );
                Error::new(ErrorKind::Other, "查询试卷题目失败")
            })?
    };

    // 4. 组装数据
    Ok(to_resp(paper, paper_groups, paper_questions))
}

// 组装试卷详情返回
fn to_resp(
    paper: Paper,
    paper_groups: Vec<PaperGroup>,
    paper_questions: Vec<PaperQuestion>,
) -> PaperResp {
    // 4. 组装数据
    let mut resp = to_paper_resp(paper);

    // 5. 构建题型和题目的映射关系（优化性能）
    let mut questions_map: std::collections::HashMap<i64, Vec<PaperQuestionResp>> =
        std::collections::HashMap::new();

    for question in paper_questions {
        let group_id = question.group_id;
        let question_resp = to_paper_question_resp(question);
        questions_map
            .entry(group_id)
            .or_insert_with(Vec::new)
            .push(question_resp);
    }

    // 6. 组装最终结果
    let mut groups = Vec::with_capacity(paper_groups.len());
    for group in paper_groups {
        let mut group_resp = to_paper_group_resp(group);
        group_resp.questions = questions_map.remove(&group_resp.id).unwrap_or_default();
        groups.push(group_resp);
    }

    resp.groups = groups;

    resp
}

// 转换为 PaperResp（优化：减少 clone）
fn to_paper_resp(row: Paper) -> PaperResp {
    PaperResp {
        id: row.id,
        related_id: row.related_id,
        related_name: row.related_name,
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
        groups: Vec::new(),
        created_at: to_local_datetime(row.created_at),
        updated_at: to_local_datetime(row.updated_at),
    }
}

// 转换为 PaperGroupResp
fn to_paper_group_resp(row: PaperGroup) -> PaperGroupResp {
    PaperGroupResp {
        id: row.id,
        paper_id: row.paper_id,
        gen_id: row.gen_id,
        type_name: row.type_name,
        sub_title: row.sub_title,
        questions: Vec::new(),
    }
}

// 转换为 PaperQuestionResp
fn to_paper_question_resp(row: PaperQuestion) -> PaperQuestionResp {
    PaperQuestionResp {
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
) -> Result<PaperListResp, Error> {
    let db = &app_conf.db;
    // 检查参数
    if req.related_id <= 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "考点/学段分类不能为空"));
    }

    // 1. 构建过滤条件
    let (where_clause, param_count) = Paper::build_condition(&req);

    // 2. 查询总数
    let total = Paper::count(db, &req, &where_clause).await.map_err(|err| {
        error!("Select paper count err: {}", err);
        Error::new(ErrorKind::Other, "查询试卷总数失败")
    })?;

    // 3. 查询列表
    let papers = Paper::list(db, &req, &where_clause, param_count)
        .await
        .map_err(|err| {
            error!("Select paper list err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷列表失败")
        })?;

    // 后续还要拼接状态等
    let list: Vec<PaperResp> = papers.into_iter().map(to_paper_resp).collect();

    Ok(PaperListResp {
        list,
        page_no: req.page_no,
        page_size: req.page_size,
        total,
    })
}

// 最新试卷
pub async fn latest(app_conf: web::Data<AppConfig>, count: i64) -> Result<Vec<PaperResp>, Error> {
    let papers = Paper::get_latest_papers(&app_conf.db, count)
        .await
        .map_err(|err| {
            error!("Select paper list err: {}", err);
            Error::new(ErrorKind::Other, "查询试卷列表失败")
        })?;

    // 后续还要拼接状态等
    let list: Vec<PaperResp> = papers.into_iter().map(to_paper_resp).collect();

    Ok(list)
}
