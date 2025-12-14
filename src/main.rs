mod api;
mod config;
mod constant;
mod route;
mod service;
mod util;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use std::path::PathBuf;

/// 命令行参数结构
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// 元数据路径 命令行实际输入时下划线需要转化为中横线, 例如 open-tiku-api --meta-path /home/zhangguangxun/Public/open-tiku-meta
    #[clap(long, default_value = "/home/zhangguangxun/Public/open-tiku-meta")]
    meta_path: PathBuf,

    /// 监听地址
    #[clap(long, default_value = "127.0.0.1")]
    host: String,

    /// 监听端口
    #[clap(long, default_value = "8082")]
    port: u16,
}

// 应用配置
#[derive(Clone)]
pub struct AppConfig {
    meta_path: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let args = Args::parse();
    let app_conf = AppConfig {
        meta_path: args.meta_path,
    };
    let addr = format!("{}:{}", args.host, args.port);

    // log
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        // app
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(app_conf.clone()))
            .service(web::scope("/config").configure(route::config))
            .service(web::scope("/file").configure(route::file))
            .service(web::scope("/question").configure(route::question))
            .service(web::scope("/edit").configure(route::edit))
    })
    .bind(&addr)?
    .run()
    .await
}
