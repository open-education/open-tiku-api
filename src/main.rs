mod api;
mod app;
mod constant;
mod enums;
mod middleware;
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
    // db
    database_url: String,

    // server
    server_host: String,
    server_port: u16,

    // meta
    meta_path: String,

    // github
    github_client_id: String,
    github_client_secret: String,
    github_redirect_uri: String,

    // qq
    qq_client_id: String,
    qq_client_secret: String,
    qq_redirect_uri: String,

    // homepage
    website_home_url: String,

    // oauth secret
    oauth_state_secret: String,

    // student pepper
    student_pepper: String,

    // smtp email 服务配置
    smtp_server: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    smtp_from_name: String,
    smtp_from_email: String,
}

// 应用配置
#[derive(Clone)]
pub struct AppConfig {
    db: PgPool,                                          // 数据库连接池
    meta_path: String,                                   // 元数据存储根目录
    github: (String, String, String),                    // GitHub (client_id, secret, redirect_uri)
    qq: (String, String, String),                        // qq (client_id, secret, redirect_uri)
    website_home_url: String,                            // 网站首页
    oauth_state_secret: String,                          // 第三方登录校验 secret
    student_pepper: String,                              // 学生账户胡椒值
    smtp: (String, u16, String, String, String, String), // smtp(server, port, username, password, from_name, from_email)
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
