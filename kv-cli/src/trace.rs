use log::LevelFilter;
use std::io::BufWriter;
use std::io::Write;
use std::str::FromStr;

use anyhow::Result;
use tracing_appender::rolling::RollingFileAppender;
use tracing_appender::rolling::Rotation;

const MAX_LOG_FILES: usize = 10;

//! log info
#[allow(dyn_drop)]
pub async fn init_logging(
    dir: &str,
    level: &str,
) -> Result<Vec<Box<dyn Drop + Send + Sync + 'static>>> {
    let mut guards: Vec<Box<dyn Drop + Send + Sync + 'static>> = Vec::new();
    let mut logger = fern::Dispatch::new();

    let rolling = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("kvcli.log")
        .max_log_files(MAX_LOG_FILES)
        .build(dir)?;
    let (non_blocking, flush_guard) = tracing_appender::non_blocking(rolling);
    let buffered_non_blocking = BufWriter::with_capacity(64 * 1024 * 1024, non_blocking);

    guards.push(Box::new(flush_guard));
    let dispatch_file = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] - {} - [{}] {}",
                chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::from_str(level)?)
        .chain(Box::new(buffered_non_blocking) as Box<dyn Write + Send>);
    logger = logger.chain(dispatch_file);

    let dispatch_stderr = fern::Dispatch::new()
        .level(LevelFilter::Warn)
        .filter(|metadata| metadata.target() == "server_warnings")
        .format(|out, message, _| {
            out.finish(format_args!(
                "\x1B[{}m{}\x1B[0m",
                fern::colors::Color::Yellow.to_fg_str(),
                message
            ))
        })
        .chain(std::io::stderr());
    logger = logger.chain(dispatch_stderr);

    if logger.apply().is_err() {
        eprintln!("logger has already been set");
        return Ok(Vec::new());
    }

    Ok(guards)
}
