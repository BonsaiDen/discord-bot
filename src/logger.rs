// External Dependencies ------------------------------------------------------
use log;
use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};
use chrono;


// Logger Implementation ------------------------------------------------------
pub struct Logger;

impl Logger {

    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Trace);
            Box::new(Logger)
        })
    }

}

impl log::Log for Logger {

    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("[{}] [{}] {}", chrono::Local::now(), record.level(), record.args());
        }
    }

}

