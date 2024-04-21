use log::{Metadata, Record, Level};
use core::fmt::Arguments;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger
    log::set_max_level(log::LevelFilter::Trace);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        if self.enabled(record.metadata()) {
            let level = record.level();
            let args = record.args();
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);
            // let args = format_args!("[{}:{}] {}", file, line, args);
            match level {
                Level::Error=>error(format_args!("[{}:{}] {}", file, line, args)),
                Level::Warn=>warn(format_args!("[{}:{}] {}", file, line, args)),
                Level::Info=>info(format_args!("[{}:{}] {}", file, line, args)),
                _=>println!("[{}] {}",level,format_args!("[{}:{}] {}", file, line, args))
            };
        }
    }

    fn flush(&self) {}
}
fn info(args: Arguments){
    println!("\x1b[32mINFO:\x1b[0m{}",args);
}
fn warn(args: Arguments){
    println!("\x1b[33;1;4mWARNING\x1b[0;33;1m:{}",args);
}
fn error(args: Arguments){
    println!("\x1b[31;1mERROR:{}\x1b[0m",args);
}