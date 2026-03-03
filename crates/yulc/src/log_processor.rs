
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: NaiveDateTime,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level_str = parts[1].trim();
        let module = parts[2].trim();
        let message = parts[3].trim();

        let timestamp = match NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S") {
            Ok(ts) => ts,
            Err(_) => return None,
        };

        let level = match level_str {
            "INFO" => LogLevel::INFO,
            "WARN" => LogLevel::WARN,
            "ERROR" => LogLevel::ERROR,
            "DEBUG" => LogLevel::DEBUG,
            _ => return None,
        };

        Some(LogEntry {
            timestamp,
            level,
            module: module.to_string(),
            message: message.to_string(),
        })
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| match (&entry.level, &level) {
                (LogLevel::ERROR, LogLevel::ERROR) => true,
                (LogLevel::WARN, LogLevel::WARN) => true,
                (LogLevel::INFO, LogLevel::INFO) => true,
                (LogLevel::DEBUG, LogLevel::DEBUG) => true,
                _ => false,
            })
            .collect()
    }

    pub fn filter_by_module(&self, module_pattern: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module.contains(module_pattern))
            .collect()
    }

    pub fn get_error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.level, LogLevel::ERROR))
            .count()
    }

    pub fn get_earliest_timestamp(&self) -> Option<NaiveDateTime> {
        self.entries
            .iter()
            .map(|entry| entry.timestamp)
            .min()
    }

    pub fn get_latest_timestamp(&self) -> Option<NaiveDateTime> {
        self.entries
            .iter()
            .map(|entry| entry.timestamp)
            .max()
    }
}