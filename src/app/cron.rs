use crate::app::config;
use crate::task;

/// 运行定时任务入口
/// 启动方式类似:
/// ./open-tiku-api question-upload // 上传题目
pub async fn run_cron(args: Vec<String>) {
    let task_name = args.get(2).expect("需要指定任务名称");

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 定时任务不需要监听端口这部分配置无需关注
    let (_, app_config) = config::init().await;

    // 将任务名称注册到匹配条件中
    match task_name.as_str() {
        "question-upload" => task::question::upload(&app_config).await,
        "cleanup-session" => task::session::cleanup(&app_config).await,
        _ => {
            eprintln!("未知任务: {}", task_name);
            std::process::exit(1);
        }
    }
}
