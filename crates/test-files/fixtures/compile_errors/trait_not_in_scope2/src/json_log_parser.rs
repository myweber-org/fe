use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq)]
enum LogSeverity {
    Error,
    Warning,
    Info,
    Debug,
}

impl LogSeverity {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogSeverity::Error),
            "warning" => Some(LogSeverity::Warning),
            "info" => Some(LogSeverity::Info),
            "debug" => Some(LogSeverity::Debug),
            _ => None,
        }
    }
}

pub fn filter_logs_by_severity(file_path: &str, severity: LogSeverity) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
            if let Some(level) = json_value.get("level").and_then(|v| v.as_str()) {
                if let Some(log_severity) = LogSeverity::from_str(level) {
                    if log_severity == severity {
                        filtered_logs.push(line);
                    }
                }
            }
        }
    }

    Ok(filtered_logs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_error_logs() {
        let log_data = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "error", "message": "Something went wrong"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "info", "message": "System started"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "error", "message": "Another error occurred"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let result = filter_logs_by_severity(temp_file.path().to_str().unwrap(), LogSeverity::Error).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("Something went wrong"));
        assert!(result[1].contains("Another error occurred"));
    }

    #[test]
    fn test_severity_parsing() {
        assert_eq!(LogSeverity::from_str("error"), Some(LogSeverity::Error));
        assert_eq!(LogSeverity::from_str("ERROR"), Some(LogSeverity::Error));
        assert_eq!(LogSeverity::from_str("warning"), Some(LogSeverity::Warning));
        assert_eq!(LogSeverity::from_str("info"), Some(LogSeverity::Info));
        assert_eq!(LogSeverity::from_str("debug"), Some(LogSeverity::Debug));
        assert_eq!(LogSeverity::from_str("unknown"), None);
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
    pub component: String,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse_logs(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(log_entry) => logs.push(log_entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let logs = self.parse_logs()?;
        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| log.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn filter_by_time_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let logs = self.parse_logs()?;
        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| log.timestamp >= start_time && log.timestamp <= end_time)
            .collect();
        Ok(filtered)
    }

    pub fn count_logs_by_component(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let logs = self.parse_logs()?;
        let mut counts = std::collections::HashMap::new();

        for log in logs {
            *counts.entry(log.component.clone()).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_log_entry() -> LogEntry {
        LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            component: "api".to_string(),
        }
    }

    #[test]
    fn test_log_entry_serialization() {
        let log = create_test_log_entry();
        let json = serde_json::to_string(&log).unwrap();
        let parsed: LogEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(log.level, parsed.level);
        assert_eq!(log.message, parsed.message);
        assert_eq!(log.component, parsed.component);
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::INFO, LogLevel::INFO);
        assert_ne!(LogLevel::INFO, LogLevel::ERROR);
    }
}