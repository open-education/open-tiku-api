use crate::app::conf::CronAppState;
use crate::model::class_student::ClassStudent;
use crate::model::paper::Paper;
use crate::model::question::{AuthorQuestion, Question};
use crate::model::stat::{CountInfo, Stat};
use crate::model::textbook::Textbook;
use crate::model::user_identity::UserIdentity;
use crate::service::user::get_user_map;
use crate::util::error::AppError;
use crate::util::local::get_today;
use sqlx::PgPool;
use sqlx::types::Json;
use tracing::error;

// 数据统计
// 一些条件没有针对性建立索引所以存在性能问题, 后面如果整体比较慢可以考虑建立一些索引
pub async fn stat(conf: &CronAppState) {
    let date = get_today();
    let db = &conf.db;

    // 获取已有的统计数据, 没有则初始化, 因为可能被其他入口初始化过不可丢失其他值
    let mut row = Stat::find_by_id(db, date)
        .await
        .expect("获取统计数据出错")
        .unwrap_or_else(|| Stat {
            date: Some(date),
            ..Default::default()
        });

    // 网站统计
    let count_info = count_info(db).await.unwrap_or_else(|e| {
        error!("stat count info err: {}", e.msg);
        CountInfo::default()
    });
    row.count_info = Json(count_info);

    // 最新上传题目
    let latest_question_ids = Question::latest_question_ids(db).await.unwrap_or_else(|e| {
        error!("stat latest question ids err: {e}");
        Vec::new()
    });
    row.latest_question_ids = Json(latest_question_ids);

    // 热门题目-做题最多
    // 热门教材-题目最多
    let top_textbooks = Question::find_top_textbooks(db).await.unwrap_or_else(|e| {
        error!("stat top textbook num err: {e}");
        Vec::new()
    });
    row.top_textbook_info = Json(top_textbooks);

    // 活跃教师-上传题目最多
    let author_questions = author_top_question(db).await.unwrap_or_else(|e| {
        error!("stat author question err: {}", e.msg);
        Vec::new()
    });
    row.top_teacher_info = Json(author_questions);

    // 活跃学生-做题最多

    // 保存统计数据
    Stat::save(db, row).await.expect("task 保存统计数据出错")
}

// 传题最多作者
async fn author_top_question(pool: &PgPool) -> Result<Vec<AuthorQuestion>, AppError> {
    let mut rows = Question::find_author_top_question(pool)
        .await
        .unwrap_or_else(|e| {
            error!("stat author question err: {}", e);
            Vec::new()
        });
    let author_ids: Vec<i64> = rows.iter().map(|item| item.author_id).collect();
    let author_name_map = get_user_map(pool, author_ids).await?;
    for row in &mut rows {
        let name = author_name_map
            .get(&row.author_id)
            .cloned()
            .unwrap_or_default();
        row.author_name = name;
    }

    Ok(rows)
}

// 网站统计
async fn count_info(pool: &PgPool) -> Result<CountInfo, AppError> {
    // 教材总数
    let textbook_num = Textbook::find_textbook_num(pool).await.map_err(|e| {
        error!("get text book_num err: {e}");
        AppError::db_error("统计教材数量出错")
    })?;

    // 题目总数
    let question_num = Question::find_question_num(pool).await.map_err(|e| {
        error!("get question_num err: {e}");
        AppError::db_error("统计题目数量出错")
    })?;

    // 试卷套数
    let paper_num = Paper::find_paper_num(pool).await.map_err(|e| {
        error!("get paper_num err: {e}");
        AppError::db_error("统计试卷数量出错")
    })?;
    // 教师人数
    let teacher_num = UserIdentity::count(pool).await.map_err(|e| {
        error!("get teacher num err: {e}");
        AppError::db_error("统计教师数量出错")
    })?;
    // 学生人数
    let student_num = ClassStudent::find_student_num(pool).await.map_err(|e| {
        error!("get student num err: {e}");
        AppError::db_error("统计学生数量出错")
    })?;

    Ok(CountInfo {
        textbook_num,
        question_num,
        paper_num,
        teacher_num,
        student_num,
    })
}
