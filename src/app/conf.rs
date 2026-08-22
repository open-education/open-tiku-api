use config::{Config, File, FileFormat};
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

/// 配置相关初始化

// 配置结构定义

// 服务监听地址和端口配置
#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

// 数据库连接信息
#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub web_max_connections: u32,
    pub task_max_connections: u32,
}

// 图片等资源存储路径
#[derive(Deserialize, Clone)]
pub struct MetaConfig {
    pub path: String,
}

// GitHub 登录认证配置
#[derive(Deserialize, Clone)]
pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

// QQ 登录认证配置
#[derive(Deserialize, Clone)]
pub struct QqConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

// 登录相关
#[derive(Deserialize, Clone)]
pub struct LoginConfig {
    pub website_home_url: String,
    pub oauth_state_secret: String,
    pub student_pepper: String,
    pub student_private_key_pem: String,
    pub github: GitHubConfig,
    pub qq: QqConfig,
}

// SMTP 邮箱发送服务配置
#[derive(Deserialize, Clone)]
pub struct SmtpEmailConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_name: String,
    pub from_email: String,
}

// 应用配置文件
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub meta: MetaConfig,
    pub login: LoginConfig,
    pub smtp: SmtpEmailConfig,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
}

// 公共初始化配置函数
// 目前 web cron 服务共用一个数据库连接池, 后续有变更再拆分
pub async fn init(is_task: bool) -> AppState {
    let builder = Config::builder()
        .add_source(File::new("config", FileFormat::Toml))
        // 新增文件是覆盖关系
        .build()
        .expect("Failed to read config.toml");

    // 反序列化成应用配置文件
    let config: AppConfig = builder
        .try_deserialize()
        .expect("Failed to read environment variable configuration");

    // 初始化数据库连接池
    let options = PgConnectOptions::from_str(&config.database.url)
        .expect("database url format is incorrect")
        .options([("timezone", "Asia/Shanghai")]);

    let pool = PgPoolOptions::new()
        .max_connections(if is_task {
            config.database.task_max_connections
        } else {
            config.database.web_max_connections
        }) // 连接池最大数量 task 需要单独控制, 通常较小
        .connect_with(options)
        .await
        .expect("Failed to connect to database");

    AppState { config, db: pool }
}
