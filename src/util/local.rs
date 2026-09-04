use crate::util::error::AppError;
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};

// 格式化本地日期时间
pub fn to_local_datetime(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

// 格式化本地日期
pub fn to_local_date(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%Y-%m-%d").to_string()
}

// 将字符串的日期 yyyy-mm-dd 转为时间戳格式
pub fn get_datetime(date_str: &str) -> Result<DateTime<Utc>, AppError> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::param_error("日期格式错误"))?;

    // 补全当天深夜的时间 (23:59:59)
    let naive_datetime = naive_date
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| AppError::param_error("时间溢出或无效时间"))?;

    // 指定为 Utc 时区并返回
    let deadline: DateTime<Utc> = Utc.from_utc_datetime(&naive_datetime);

    Ok(deadline)
}
