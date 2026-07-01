mod api;
mod app;
mod constant;
mod model;
mod service;
mod task;
mod util;

use crate::app::cron::run_cron;
use crate::app::web::run_web;
use crate::util::snowflake::init_snowflake;
use serde::Deserialize;
use sqlx::PgPool;
use std::env;

// 服务相关环境变量配置
#[derive(Deserialize)]
struct EnvConfig {
    database_url: String,
    server_host: String,
    server_port: u16,
    meta_path: String,
    github_client_id: String,
    github_client_secret: String,
    website_home_url: String,
}

// 应用配置
#[derive(Clone)]
pub struct AppConfig {
    db: PgPool,               // 数据库连接池
    meta_path: String,        // 元数据存储根目录
    github: (String, String), // (client_id, secret)
    website_home_url: String, // 网站首页
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // 初始化id生成器
    init_snowflake(10);

    // 需要手动传入该参数才能确定为执行定时任务
    // ./open-tiku-api task [...](具体要执行的任务名称, 配置在run_cron()方法入口中)
    if args.len() > 1 && args[1] == "task" {
        // 任务本身不返回任何信息, 任务内部去处理, 仅将控制台输入的参数传入
        run_cron(args).await;
        return Ok(());
    }

    // 默认启动 web 服务
    run_web().await
}
