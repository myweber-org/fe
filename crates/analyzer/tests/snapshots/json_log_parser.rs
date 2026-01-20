use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            
            if self.filter_entry(&entry) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
            return false;
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        true
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
            LogLevel::FATAL => 4,
        };
        let other_val = match other {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
            LogLevel::FATAL => 4,
        };
        Some(self_val.cmp(&other_val))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let test_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"boot","metadata":null}
{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","message":"High memory usage","component":"monitor","metadata":{"usage":85}}
{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","message":"Database connection failed","component":"db","metadata":{"retries":3}}"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_log).unwrap();

        let parser = LogParser::new(LogLevel::WARN);
        let entries = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::WARN);
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_time_range_filter() {
        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 31, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 10, 32, 0).unwrap();

        let test_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Before range","component":"test", "metadata":null}
{"timestamp":"2024-01-15T10:31:30Z","level":"INFO","message":"In range","component":"test", "metadata":null}
{"timestamp":"2024-01-15T10:32:30Z","level":"INFO","message":"After range","component":"test", "metadata":null}"#;

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_log).unwrap();

        let parser = LogParser::new(LogLevel::INFO)
            .with_time_range(start, end);
        
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "In range");
    }
}