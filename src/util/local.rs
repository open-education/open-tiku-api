use crate::util::error::AppError;
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};

// 本地日期时间

const BJ_OFFSET: FixedOffset = match FixedOffset::east_opt(8 * 3600) {
    Some(offset) => offset,
    None => panic!("无效的时区偏移"),
};

// 格式化本地日期时间
pub fn to_local_datetime(dt: Option<DateTime<Utc>>) -> String {
    dt.map(|t| {
        t.with_timezone(&BJ_OFFSET)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    })
    .unwrap_or_default()
}

// 格式化本地日期
pub fn to_local_date(dt: Option<DateTime<Utc>>) -> String {
    dt.map(|t| t.with_timezone(&BJ_OFFSET).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

// 将字符串的日期 yyyy-mm-dd 转为时间戳格式
pub fn get_datetime(date_str: &str) -> Result<DateTime<Utc>, AppError> {
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::param_error("日期格式错误"))?;

    let naive_datetime = naive_date
        .and_hms_opt(23, 59, 59)
        .ok_or_else(|| AppError::param_error("时间溢出或无效时间"))?;

    let local_datetime = BJ_OFFSET
        .from_local_datetime(&naive_datetime)
        .single()
        .ok_or_else(|| AppError::param_error("无效的北京时间"))?;

    Ok(local_datetime.with_timezone(&Utc))
}
