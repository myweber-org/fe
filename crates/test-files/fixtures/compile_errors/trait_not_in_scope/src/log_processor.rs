
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{DateTime, Utc};
use regex::Regex;

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

pub struct LogProcessor {
    pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Self {
        let pattern = Regex::new(r"\[(?P<timestamp>[\d-]+T[\d:.]+Z)\] (?P<level>\w+): (?P<message>.+)")
            .expect("Invalid regex pattern");
        LogProcessor { pattern }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).map(|caps| {
            let timestamp_str = caps.name("timestamp").unwrap().as_str();
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .unwrap()
                .with_timezone(&Utc);
            
            LogEntry {
                timestamp,
                level: caps.name("level").unwrap().as_str().to_string(),
                message: caps.name("message").unwrap().as_str().to_string(),
            }
        })
    }

    pub fn filter_by_level<'a>(&self, entries: &'a [LogEntry], level: &str) -> Vec<&'a LogEntry> {
        entries.iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn read_log_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            if let Ok(line_content) = line {
                if let Some(entry) = self.parse_line(&line_content) {
                    entries.push(entry);
                }
            }
        }
        
        Ok(entries)
    }

    pub fn find_entries_in_range(&self, entries: &[LogEntry], start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parse_valid_line() {
        let processor = LogProcessor::new();
        let line = "[2024-01-15T10:30:45Z] INFO: System started successfully";
        
        let entry = processor.parse_line(line).unwrap();
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "System started successfully");
    }

    #[test]
    fn test_filter_by_level() {
        let processor = LogProcessor::new();
        let entries = vec![
            LogEntry {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 45).unwrap(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
            },
            LogEntry {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 31, 0).unwrap(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
            },
        ];

        let filtered = processor.filter_by_level(&entries, "ERROR");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}