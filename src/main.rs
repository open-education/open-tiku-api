mod api;
mod config;
mod constant;
mod model;
mod route;
mod service;
mod util;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use envy::from_env;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

// 服务相关环境变量配置
#[derive(Deserialize)]
struct EnvConfig {
    database_url: String,
    server_host: String,
    server_port: u16,
    meta_path: String,
}

// 应用配置
#[derive(Clone)]
pub struct AppConfig {
    meta_path: String,
    db: PgPool, // 数据库连接池
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // log init
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 加载 .env 文件如果存在
    dotenv().ok();
    let env_config: EnvConfig =
        from_env::<EnvConfig>().expect("Failed to parse environment variable configuration");

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(env_config.database_url.as_str())
        .await
        .expect("Unable to connect to the database");

    // 准备应用配置信息
    let app_conf = AppConfig {
        meta_path: env_config.meta_path,
        db: pool,
    };

    // 服务监听地址和端口
    let addr = format!("{}:{}", env_config.server_host, env_config.server_port);

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
            .service(web::scope("/textbook").configure(route::textbook))
            .service(web::scope("/chapter-knowledge").configure(route::chapter_knowledge))
            .service(web::scope("/question-cate").configure(route::question_cate))
            .service(web::scope("/textbook/dict").configure(route::textbook_dict))
    })
    .bind(&addr)?
    .run()
    .await
}
