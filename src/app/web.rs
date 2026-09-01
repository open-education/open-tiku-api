use crate::app::conf;
use crate::app::log::init_logger;
use crate::app::route;
use crate::middleware::user::auth;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};
use tracing_actix_web::TracingLogger;

/// web 服务入口

pub async fn run_web() -> std::io::Result<()> {
    let guard = init_logger("app.log");
    Box::leak(Box::new(guard));

    let app_state = conf::init(false).await;

    let addr = format!(
        "{}:{}",
        app_state.config.server.host, app_state.config.server.port
    );

    // 注意路由匹配按定义顺序匹配, 所以更具体的路由需要配置在前面
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(from_fn(auth))
            .app_data(web::Data::new(app_state.clone()))
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
            .service(web::scope("/callback").configure(route::callback))
            .service(web::scope("/user").configure(route::user))
            .service(web::scope("/class/student").configure(route::class_student))
            .service(web::scope("/class").configure(route::class))
            .service(web::scope("/homework").configure(route::homework))
            .service(web::scope("/test").configure(route::test))
    })
    .bind(&addr)?
    .run()
    .await
}
