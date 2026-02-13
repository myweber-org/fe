use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    JsonError(JsonError),
    InvalidLogFormat(String),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<JsonError> for ParseError {
    fn from(err: JsonError) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: Value,
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        match parse_log_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, ParseError> {
    let json_value: Value = serde_json::from_str(line)?;
    
    let timestamp = json_value["timestamp"]
        .as_str()
        .ok_or_else(|| ParseError::InvalidLogFormat("Missing timestamp field".to_string()))?
        .to_string();
    
    let level = json_value["level"]
        .as_str()
        .ok_or_else(|| ParseError::InvalidLogFormat("Missing level field".to_string()))?
        .to_string();
    
    let message = json_value["message"]
        .as_str()
        .ok_or_else(|| ParseError::InvalidLogFormat("Missing message field".to_string()))?
        .to_string();
    
    Ok(LogEntry {
        timestamp,
        level,
        message,
        fields: json_value,
    })
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let json_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","pid":1234}"#;
        let entry = parse_log_line(json_log).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Service started");
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_json = "not a json";
        let result = parse_log_line(invalid_json);
        assert!(matches!(result, Err(ParseError::JsonError(_))));
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
                fields: Value::Null,
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
                fields: Value::Null,
            },
        ];
        
        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
    }

    #[test]
    fn test_parse_log_file() -> Result<(), ParseError> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Line 1"}}"#)?;
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","message":"Line 2"}}"#)?;
        
        let entries = parse_log_file(temp_file.path())?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "WARN");
        
        Ok(())
    }
}use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
    pub source: String,
}

pub struct LogParser {
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn filter_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }

    pub fn count_by_level(&self) -> std::collections::HashMap<LogLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    pub fn find_errors_with_keyword(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::ERROR && entry.message.contains(keyword))
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
        
        let entry1 = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::INFO,
            message: "System started".to_string(),
            source: "system".to_string(),
        };
        
        let entry2 = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::ERROR,
            message: "Disk full".to_string(),
            source: "storage".to_string(),
        };
        
        parser.entries.push(entry1);
        parser.entries.push(entry2);
        
        let errors = parser.filter_by_level(LogLevel::ERROR);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Disk full");
    }

    #[test]
    fn test_filter_by_time_range() {
        let mut parser = LogParser::new();
        
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mid_time = Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap();
        
        let entry = LogEntry {
            timestamp: mid_time,
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            source: "test".to_string(),
        };
        
        parser.entries.push(entry);
        
        let filtered = parser.filter_by_time_range(start_time, end_time);
        assert_eq!(filtered.len(), 1);
        
        let filtered = parser.filter_by_time_range(end_time, end_time);
        assert_eq!(filtered.len(), 0);
    }
}