use simplelog::*;
use std::fs::File;

pub struct Logger;

impl Logger {
    /// Initializes the logger to write logs only to a file.
    pub fn init(log_file: &str, log_level: LevelFilter) {
        CombinedLogger::init(vec![
            WriteLogger::new(
                log_level,
                Config::default(),
                File::create(log_file).expect("Failed to create log file"),
            ),
        ])
        .expect("Failed to initialize logger");
    }
}
