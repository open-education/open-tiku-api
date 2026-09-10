use crate::util::error::AppError;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::error;

// Sqlite 实现缓存服务

pub async fn init(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS kv_cache (key TEXT PRIMARY KEY, value BLOB NOT NULL, expires_at INTEGER NOT NULL);")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_cache_expires_at ON kv_cache(expires_at);")
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get<T>(pool: &SqlitePool, key: &str) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let bytes: Vec<u8> =
        sqlx::query_scalar("SELECT value FROM kv_cache WHERE key = ? AND expires_at > ?")
            .bind(key)
            .bind(now)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                error!("Get cache key: {} database error: {}", key, e);
                AppError::db_error("缓存查询出错")
            })?
            .ok_or_else(|| AppError::not_found("缓存数据为空或已过期"))?;

    let (data, _bytes_read) =
        bincode_next::serde::decode_from_slice::<T, _>(&bytes, bincode_next::config::standard())
            .map_err(|e| {
                error!("Decode cache key: {} failed, error: {:?}", key, e);
                AppError::serde_error("缓存数据反序列出错")
            })?;

    Ok(data)
}

pub async fn set<T>(pool: &SqlitePool, key: &str, value: &T, ttl: Duration)
where
    T: Serialize,
{
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expires_at = now + ttl.as_secs() as i64;

    let binary_bytes =
        match bincode_next::serde::encode_to_vec(value, bincode_next::config::standard()) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Encode cache key: {} failed, error: {:?}", key, e);
                return;
            }
        };

    if let Err(e) =
        sqlx::query("INSERT OR REPLACE INTO kv_cache (key, value, expires_at) VALUES (?, ?, ?)")
            .bind(key)
            .bind(binary_bytes)
            .bind(expires_at)
            .execute(pool)
            .await
    {
        error!("Set cache key: {} database error: {}", key, e);
    }
}

// 通过左前缀匹配批量删除缓存
pub async fn delete_by_prefix(pool: &SqlitePool, prefix: &str) {
    // 对前缀中可能存在的特殊字符（如 % 和 _）进行转义处理
    // 防止用户传入的特殊字符被 SQLite 误当成通配符去模糊匹配中间的数据
    let escaped_prefix = prefix
        .replace('/', "//")
        .replace('%', "/%")
        .replace('_', "/_");

    // 右通配符模式
    let pattern = format!("{}%", escaped_prefix);

    // SQL 层面使用 ESCAPE '/' 强行规定斜杠为转义符
    if let Err(e) = sqlx::query("DELETE FROM kv_cache WHERE key LIKE ? ESCAPE '/'")
        .bind(pattern)
        .execute(pool)
        .await
    {
        error!("Delete cache by prefix: {} database error: {}", prefix, e);
    }
}
