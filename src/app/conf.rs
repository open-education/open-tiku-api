use config::{Config, File, FileFormat};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{PgPool, SqlitePool};
use std::str::FromStr;
use std::time::Duration;

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

// Sqlite 文件配置, 连接池暂时用默认
#[derive(Deserialize, Clone)]
pub struct SqliteConfig {
    pub db_path: String,
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
// #[serde(rename = "server")] 对应 toml 中的 [server]
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub server: ServerConfig,

    pub database: DatabaseConfig,

    pub sqlite: SqliteConfig,

    pub meta: MetaConfig,

    pub login: LoginConfig,

    pub smtp: SmtpEmailConfig,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: PgPool,
    pub sqlite: SqlitePool,
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
    let db_pool = PgPoolOptions::new()
        .max_connections(if is_task {
            config.database.task_max_connections
        } else {
            config.database.web_max_connections
        }) // 连接池最大数量 task 需要单独控制, 通常较小
        .connect_with(options)
        .await
        .expect("Failed to connect to database");

    // 使用 sqlx 内建的 sqlite
    let sqlite_connection_options = SqliteConnectOptions::from_str(&config.sqlite.db_path)
        .expect("Invalid SQLite connection string")
        // 文件不存在 SQLx 会自动新建
        .create_if_missing(true)
        // 开启 WAL 模式 get 读操作永远不会被 set 写操作阻塞
        .journal_mode(SqliteJournalMode::Wal)
        // 高并发遭遇文件锁时 驱动自动排队等待 5 秒 而不是直接抛出 "database is locked" 错误
        .busy_timeout(Duration::from_secs(5));
    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        // 等待连接池分配连接的超时时间
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(sqlite_connection_options)
        .await
        .expect("Failed to create sqlx sqlite pool");

    AppState {
        config,
        db: db_pool,
        sqlite: sqlite_pool,
    }
}
