use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub use log::{debug as verbose, error};
// Log levels for use with NewLogger.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Silent,
    Error,
    Verbose,
}

pub fn init_logger(level: LogLevel, prepend: &str) {
    let level = match level {
        LogLevel::Verbose => LevelFilter::Debug,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Silent => LevelFilter::Off,
    };
    let prepend = prepend.to_string();
    Builder::new()
        .format(move |buf, record| {
            let timestamp = buf.timestamp();
            writeln!(
                buf,
                "[{}] {}: {} {}",
                timestamp,
                record.level(),
                prepend,
                record.args()
            )
        })
        .filter_level(level)
        .init();
}
