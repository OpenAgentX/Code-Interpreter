
use colored::control::ShouldColorize;
use time::macros::format_description;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{
    fmt,
    fmt::time::OffsetTime,
    Layer,
    layer::SubscriberExt
};

pub fn setup() -> Option<WorkerGuard> {
    // local time
    let offset = clia_local_offset::current_local_offset().expect("Can not get local offset!");
    let timer =
        OffsetTime::new(offset, format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"));

    let file_filter = || {
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("h2=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive("rustls=warn".parse().unwrap())
            .add_directive("sled=warn".parse().unwrap())
            .add_directive("rustyline=warn".parse().unwrap())
            .add_directive("code_interpreter=debug".parse().unwrap())
    };

    let file_appender = rolling::never("logs", "app.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_timer(timer.clone())
        .with_ansi(false)
        .with_writer(non_blocking_appender
        ).with_filter(file_filter());

    let console_filter =
        || tracing_subscriber::EnvFilter::from_default_env().add_directive("code_interpreter=warn".parse().unwrap());

    let console_layer = fmt::layer()
        .with_timer(timer.clone())
        // Only use ANSI if we should colorize
        .with_ansi(ShouldColorize::from_env().should_colorize())
        .with_span_events(fmt::format::FmtSpan::NEW)
        .with_filter(console_filter());

    let reg = tracing_subscriber::registry()
        // .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    let _map_err =
        tracing::subscriber::set_global_default(reg).map_err(|_err| eprintln!("Unable to set global default subscriber"));

    Some(_guard)
}
