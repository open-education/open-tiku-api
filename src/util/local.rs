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

// 相对时间描述
pub fn to_time_ago(dt: Option<DateTime<Utc>>) -> String {
    let Some(dt) = dt else {
        return String::from("未知");
    };

    let secs = (Utc::now() - dt).num_seconds().max(0);

    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    if secs < MINUTE {
        return String::from("1分钟前");
    }
    if secs < HOUR {
        return format!("{}分钟前", secs / MINUTE);
    }
    if secs < DAY {
        let hours = secs / HOUR;
        let mins = (secs % HOUR) / MINUTE;
        return if mins == 0 {
            format!("{hours}小时前")
        } else {
            format!("{hours}小时{mins}分钟前")
        };
    }
    if secs < MONTH {
        return format!("{}天前", ceil_div(secs, DAY));
    }
    if secs < YEAR {
        return format!("{}月前", ceil_div(secs, MONTH));
    }
    format!("{}年前", ceil_div(secs, YEAR))
}

#[inline]
fn ceil_div(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}
