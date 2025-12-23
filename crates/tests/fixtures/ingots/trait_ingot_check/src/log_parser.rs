
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: NaiveDateTime,
    level: String,
    message: String,
    source: String,
}

pub struct LogParser {
    error_pattern: Regex,
    timestamp_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            error_pattern: Regex::new(r"ERROR|FATAL|CRITICAL").unwrap(),
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
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
        if !self.error_pattern.is_match(line) {
            return None;
        }

        let timestamp = self.extract_timestamp(line)?;
        let level = self.extract_log_level(line)?;
        let source = self.extract_source(line).unwrap_or_else(|| "unknown".to_string());
        let message = self.extract_message(line);

        Some(LogEntry {
            timestamp,
            level,
            message,
            source,
        })
    }

    fn extract_timestamp(&self, line: &str) -> Option<NaiveDateTime> {
        self.timestamp_pattern
            .find(line)
            .and_then(|m| NaiveDateTime::parse_from_str(m.as_str(), "%Y-%m-%d %H:%M:%S").ok())
    }

    fn extract_log_level(&self, line: &str) -> Option<String> {
        let patterns = ["ERROR", "FATAL", "CRITICAL"];
        patterns
            .iter()
            .find(|&&pattern| line.contains(pattern))
            .map(|&s| s.to_string())
    }

    fn extract_source(&self, line: &str) -> Option<String> {
        let module_pattern = Regex::new(r"\[([^\]]+)\]").unwrap();
        module_pattern
            .captures(line)
            .map(|caps| caps[1].to_string())
    }

    fn extract_message(&self, line: &str) -> String {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() > 3 {
            parts[3..].join(" ")
        } else {
            line.to_string()
        }
    }
}

pub fn filter_by_timeframe(
    entries: &[LogEntry],
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_error_line() {
        let parser = LogParser::new();
        let line = "2024-01-15 14:30:25 [database] ERROR: Connection timeout";
        
        let entry = parser.parse_line(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.source, "database");
        assert_eq!(entry.message, "Connection timeout");
    }

    #[test]
    fn test_filter_timeframe() {
        let entries = vec![
            LogEntry {
                timestamp: NaiveDate::from_ymd_opt(2024, 1, 15)
                    .unwrap()
                    .and_hms_opt(10, 0, 0)
                    .unwrap(),
                level: "ERROR".to_string(),
                message: "Test 1".to_string(),
                source: "app".to_string(),
            },
            LogEntry {
                timestamp: NaiveDate::from_ymd_opt(2024, 1, 15)
                    .unwrap()
                    .and_hms_opt(14, 0, 0)
                    .unwrap(),
                level: "ERROR".to_string(),
                message: "Test 2".to_string(),
                source: "app".to_string(),
            },
        ];

        let start = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(16, 0, 0)
            .unwrap();

        let filtered = filter_by_timeframe(&entries, start, end);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].message, "Test 2");
    }
}