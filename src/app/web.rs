use crate::app::config;
use crate::app::route;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};

/// web 服务入口

pub async fn run_web() -> std::io::Result<()> {
    // 初始化日志
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let (env_config, app_config) = config::init().await;

    let addr = format!("{}:{}", env_config.server_host, env_config.server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(app_config.clone()))
            .service(web::scope("/file").configure(route::file))
            .service(web::scope("/question").configure(route::question))
            .service(web::scope("/edit").configure(route::edit))
            .service(web::scope("/textbook").configure(route::textbook))
            .service(web::scope("/chapter-knowledge").configure(route::chapter_knowledge))
            .service(web::scope("/question-cate").configure(route::question_cate))
            .service(web::scope("/other/dict").configure(route::textbook_dict))
            .service(web::scope("/task").configure(route::task))
            .service(web::scope("/paper").configure(route::paper))
            .service(web::scope("/text").configure(route::text))
    })
    .bind(&addr)?
    .run()
    .await
}
