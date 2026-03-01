use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }

    pub fn get_statistics(&self) -> Vec<(LogLevel, usize)> {
        let mut stats = vec![
            (LogLevel::INFO, 0),
            (LogLevel::WARN, 0),
            (LogLevel::ERROR, 0),
            (LogLevel::DEBUG, 0),
        ];

        for entry in &self.entries {
            match entry.level {
                LogLevel::INFO => stats[0].1 += 1,
                LogLevel::WARN => stats[1].1 += 1,
                LogLevel::ERROR => stats[2].1 += 1,
                LogLevel::DEBUG => stats[3].1 += 1,
            }
        }

        stats
    }

    pub fn search_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(query))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        let test_data = r#"
            {"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"core"}
            {"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","component":"network"}
            {"timestamp":"2024-01-15T10:32:00Z","level":"INFO","message":"User logged in","component":"auth"}
        "#;

        let temp_file = "test_logs.json";
        std::fs::write(temp_file, test_data).unwrap();

        parser.load_from_file(temp_file).unwrap();
        let info_logs = parser.filter_by_level(LogLevel::INFO);
        assert_eq!(info_logs.len(), 2);

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_time_range_filter() {
        let mut parser = LogParser::new();
        let test_data = r#"
            {"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Event 1","component":"test"}
            {"timestamp":"2024-01-15T10:35:00Z","level":"INFO","message":"Event 2","component":"test"}
            {"timestamp":"2024-01-15T10:40:00Z","level":"INFO","message":"Event 3","component":"test"}
        "#;

        let temp_file = "test_time_logs.json";
        std::fs::write(temp_file, test_data).unwrap();

        parser.load_from_file(temp_file).unwrap();

        let start = Utc.with_ymd_and_hms(2024, 1, 15, 10, 32, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, 15, 10, 42, 0).unwrap();

        let filtered = parser.filter_by_time_range(start, end);
        assert_eq!(filtered.len(), 2);

        std::fs::remove_file(temp_file).unwrap();
    }
}