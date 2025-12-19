use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    timestamp_pattern: Regex,
    level_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}").unwrap(),
            level_pattern: Regex::new(r"\[(ERROR|WARN|INFO|DEBUG|TRACE)\]").unwrap(),
        }
    }

    pub fn parse_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
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

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let timestamp_match = self.timestamp_pattern.find(line)?;
        let level_match = self.level_pattern.find(line)?;

        let timestamp_str = timestamp_match.as_str();
        let level_str = &line[level_match.start() + 1..level_match.end() - 1];

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;
        let message = line[level_match.end()..].trim().to_string();

        Some(LogEntry {
            timestamp,
            level: level_str.to_string(),
            message,
        })
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn find_errors(&self, entries: &[LogEntry]) -> Vec<&LogEntry> {
        self.filter_by_level(entries, "ERROR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new();
        let line = "2024-01-15T10:30:45+00:00 [ERROR] Database connection failed";
        
        let entry = parser.parse_line(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_filter_errors() {
        let parser = LogParser::new();
        let entries = vec![
            LogEntry {
                timestamp: DateTime::parse_from_rfc3339("2024-01-15T10:30:45+00:00").unwrap(),
                level: "ERROR".to_string(),
                message: "DB fail".to_string(),
            },
            LogEntry {
                timestamp: DateTime::parse_from_rfc3339("2024-01-15T10:31:00+00:00").unwrap(),
                level: "INFO".to_string(),
                message: "Started".to_string(),
            },
        ];

        let errors = parser.find_errors(&entries);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}