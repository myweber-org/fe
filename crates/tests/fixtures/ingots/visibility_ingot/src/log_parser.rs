
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+): (?P<message>.+)")?;
        Ok(LogParser { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).and_then(|caps| {
            let timestamp_str = caps.name("timestamp")?.as_str();
            let level = caps.name("level")?.as_str().to_string();
            let message = caps.name("message")?.as_str().to_string();

            DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z")
                .ok()
                .map(|timestamp| LogEntry {
                    timestamp,
                    level,
                    message,
                })
        })
    }

    pub fn filter_by_level<'a>(
        &self,
        entries: &'a [LogEntry],
        level_filter: &str,
    ) -> Vec<&'a LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level_filter.to_lowercase())
            .collect()
    }

    pub fn read_log_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new().unwrap();
        let line = "[2023-10-05 14:30:25 +00:00] ERROR: Database connection failed";
        let entry = parser.parse_line(line).unwrap();

        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.timestamp.format("%Y-%m-%d").to_string(), "2023-10-05");
    }

    #[test]
    fn test_filter_entries() {
        let parser = LogParser::new().unwrap();
        let entries = vec![
            LogEntry {
                timestamp: DateTime::parse_from_str("2023-10-05 10:00:00 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
                level: "INFO".to_string(),
                message: "Server started".to_string(),
            },
            LogEntry {
                timestamp: DateTime::parse_from_str("2023-10-05 10:05:00 +00:00", "%Y-%m-%d %H:%M:%S %z").unwrap(),
                level: "ERROR".to_string(),
                message: "Connection timeout".to_string(),
            },
        ];

        let filtered = parser.filter_by_level(&entries, "error");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}