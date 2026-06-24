use chrono::{DateTime, Local, Utc};

// 格式化本地时间
pub fn to_local_datetime(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
