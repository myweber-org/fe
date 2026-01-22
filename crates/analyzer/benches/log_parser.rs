use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: NaiveDateTime,
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
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
            level_pattern: Regex::new(r"\[(ERROR|WARN|INFO|DEBUG)\]").unwrap(),
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
        let timestamp = self.timestamp_pattern.find(line)?;
        let level_capture = self.level_pattern.captures(line)?;
        
        let timestamp_str = timestamp.as_str();
        let level = level_capture.get(1)?.as_str().to_string();
        
        let message_start = std::cmp::max(timestamp.end(), level_capture.get(0)?.end()) + 1;
        let message = line[message_start..].trim().to_string();

        NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|ts| LogEntry {
                timestamp: ts,
                level,
                message,
            })
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new();
        let line = "2024-01-15 14:30:22 [ERROR] Database connection failed";
        
        let entry = parser.parse_line(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_filter_errors() {
        let parser = LogParser::new();
        let entries = vec![
            LogEntry {
                timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
            },
            LogEntry {
                timestamp: NaiveDateTime::parse_from_str("2024-01-15 10:01:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
            },
        ];

        let errors = parser.filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}