use crate::app::config;
use crate::app::log::init_logger;
use crate::task;

/// 运行定时任务入口
/// 启动方式类似:
/// ./open-tiku-api task question-upload // 上传题目
pub async fn run_cron(args: Vec<String>) {
    let task_name = args.get(2).expect("需要指定任务名称");

    init_logger();

    let app_state = config::init(true).await;

    // 将任务名称注册到匹配条件中
    match task_name.as_str() {
        "question-upload" => task::question::upload(&app_state).await,
        "session-cleanup" => task::session::cleanup(&app_state).await,
        "fix-path" => task::fix::path(&app_state).await,
        _ => {
            eprintln!("未知任务: {}", task_name);
            std::process::exit(1);
        }
    }
}
