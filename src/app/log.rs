use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// 初始化日志配置

pub fn init_logger(file_name: &str) -> WorkerGuard {
    // 日志存储位置
    let log_dir = "./log";
    let file_prefix = file_name;

    // 每天自动分割日志
    let file_appender = tracing_appender::rolling::daily(log_dir, file_prefix);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 日期偏移 8小时, 内容是本地时间, 但是日志文件名还是 UTC 时间
    let local_offset = time::UtcOffset::from_hms(8, 0, 0).expect("Valid offset");
    let format_items = time::format_description::parse_borrowed::<3>(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
    )
    .expect("Valid format");
    let timer = OffsetTime::new(local_offset, format_items);

    // 需要收集日志如果默认不输出则要手动开启
    // sqlx 的 select 等语句是 debug 级别, info 级别没有日志
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,actix_web=info,tracing_actix_web=info,sqlx=debug")
    });

    // 记录 json 格式的日志文件
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_target(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json()
        .with_current_span(true)
        .compact()
        .with_timer(timer);

    // 注册日志组件
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .init();

    _guard
}
