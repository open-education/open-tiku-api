use crate::{AppConfig, EnvConfig};
use dotenvy::dotenv;
use envy::from_env;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

/// 配置相关初始化

// 公共初始化配置函数
// 目前 web cron 服务共用一个数据库连接池, 后续有变更再拆分
pub async fn init() -> (EnvConfig, AppConfig) {
    dotenv().ok();

    let env_config: EnvConfig =
        from_env::<EnvConfig>().expect("Failed to parse environment variable configuration");

    let options = PgConnectOptions::from_str(&env_config.database_url)
        .expect("database url format is incorrect")
        .options([("timezone", "Asia/Shanghai")]);

    let pool = PgPoolOptions::new()
        .max_connections(2) // 连接池后续追加至 .env 配置文件中
        .connect_with(options)
        .await
        .expect("Unable to connect to the database");

    let app_config = AppConfig {
        db: pool,
        meta_path: env_config.meta_path.clone(),
        github: (
            env_config.github_client_id.clone(),
            env_config.github_client_secret.clone(),
        ),
        website_home_url: env_config.website_home_url.clone(),
    };

    (env_config, app_config)
}
