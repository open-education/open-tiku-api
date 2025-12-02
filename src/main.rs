mod api;
mod config;
mod constant;
mod route;
mod service;
mod util;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // log
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        // app
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(web::scope("/config").configure(route::config))
            .service(web::scope("/file").configure(route::file))
            .service(web::scope("/question").configure(route::question))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
