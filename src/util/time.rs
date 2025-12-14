use chrono::{DateTime, FixedOffset, Utc};
use log::error;

pub fn get_beijing_time_info() -> (i64, i64, String) {
    let utc_now = Utc::now();
    match FixedOffset::east_opt(8 * 3600) {
        Some(beijing_offset) => {
            let beijing_time: DateTime<FixedOffset> = utc_now.with_timezone(&beijing_offset);
            let seconds = beijing_time.timestamp();
            let millis = beijing_time.timestamp_millis();
            let formatted = beijing_time.format("%Y-%m-%d %H:%M:%S").to_string();

            (seconds, millis, formatted)
        }
        None => {
            error!("Could not get beijing time");
            (utc_now.timestamp(), utc_now.timestamp(), String::from(""))
        }
    }
}
