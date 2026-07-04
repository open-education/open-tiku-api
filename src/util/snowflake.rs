use idgen_rs::{extract_time_utc, next_id, snowflake_init};
use std::sync::OnceLock;

// 存放一个空元组，仅表示“已初始化”
static INIT: OnceLock<()> = OnceLock::new();

/// 初始化雪花 ID 生成器（必须在使用 `generate_id` 前调用）
/// 如果已初始化，重复调用不会产生任何影响（只会执行一次）。
pub fn init_snowflake(worker_id: u16) {
    INIT.get_or_init(|| {
        snowflake_init(worker_id);
    });
}

/// 生成一个唯一 ID
pub fn generate_id() -> i64 {
    // 检查是否已初始化
    if INIT.get().is_none() {
        panic!("Snowflake generator not initialized. Call `init_snowflake(worker_id)` first.");
    }
    next_id() as i64
}

/// 从 ID 中提取创建时间（UTC）
pub fn parse_id_time(id: u64) -> Option<chrono::DateTime<chrono::Utc>> {
    extract_time_utc(id)
}
