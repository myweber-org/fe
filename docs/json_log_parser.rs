
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidTimestamp(String),
    MissingField(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::JsonError(e) => write!(f, "JSON parsing error: {}", e),
            ParseError::InvalidTimestamp(s) => write!(f, "Invalid timestamp format: {}", s),
            ParseError::MissingField(s) => write!(f, "Missing required field: {}", s),
        }
    }
}

impl Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(error: serde_json::Error) -> Self {
        ParseError::JsonError(error)
    }
}

struct LogParser {
    file_path: String,
    min_level: Option<String>,
    service_filter: Option<String>,
}

impl LogParser {
    fn new(file_path: String) -> Self {
        LogParser {
            file_path,
            min_level: None,
            service_filter: None,
        }
    }

    fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_string());
        self
    }

    fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            match self.parse_line(&line_content) {
                Ok(entry) => {
                    if self.passes_filters(&entry) {
                        entries.push(entry);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, ParseError> {
        let raw_entry: serde_json::Value = serde_json::from_str(line)?;
        
        let timestamp_str = raw_entry.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("timestamp".to_string()))?;
        
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|_| ParseError::InvalidTimestamp(timestamp_str.to_string()))?
            .with_timezone(&Utc);

        let level = raw_entry.get("level")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ParseError::MissingField("level".to_string()))?;

        let service = raw_entry.get("service")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ParseError::MissingField("service".to_string()))?;

        let message = raw_entry.get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ParseError::MissingField("message".to_string()))?;

        let metadata = raw_entry.get("metadata").cloned();

        Ok(LogEntry {
            timestamp,
            level,
            service,
            message,
            metadata,
        })
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        if let Some(ref min_level) = self.min_level {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let entry_level_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_level_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            match (entry_level_idx, min_level_idx) {
                (Some(e_idx), Some(m_idx)) if e_idx < m_idx => return false,
                _ => {}
            }
        }

        if let Some(ref service_filter) = self.service_filter {
            if entry.service != *service_filter {
                return false;
            }
        }

        true
    }

    fn generate_summary(&self, entries: &[LogEntry]) -> Result<String, ParseError> {
        let mut summary = String::new();
        let mut level_counts = std::collections::HashMap::new();
        let mut service_counts = std::collections::HashMap::new();

        for entry in entries {
            *level_counts.entry(&entry.level).or_insert(0) += 1;
            *service_counts.entry(&entry.service).or_insert(0) += 1;
        }

        summary.push_str("Log Analysis Summary\n");
        summary.push_str("===================\n");
        summary.push_str(&format!("Total entries: {}\n", entries.len()));
        
        if let Some(earliest) = entries.iter().map(|e| e.timestamp).min() {
            summary.push_str(&format!("Time range start: {}\n", earliest));
        }
        
        if let Some(latest) = entries.iter().map(|e| e.timestamp).max() {
            summary.push_str(&format!("Time range end: {}\n", latest));
        }

        summary.push_str("\nLevel distribution:\n");
        for (level, count) in &level_counts {
            summary.push_str(&format!("  {}: {}\n", level, count));
        }

        summary.push_str("\nService distribution:\n");
        for (service, count) in &service_counts {
            summary.push_str(&format!("  {}: {}\n", service, count));
        }

        Ok(summary)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("logs/app.log".to_string())
        .with_min_level("info")
        .with_service_filter("api");

    let entries = parser.parse()?;
    
    if entries.is_empty() {
        println!("No log entries found matching the criteria.");
        return Ok(());
    }

    let summary = parser.generate_summary(&entries)?;
    println!("{}", summary);

    println!("\nRecent entries:");
    let recent_entries: Vec<_> = entries.iter().rev().take(5).collect();
    for entry in recent_entries {
        println!(
            "[{}] {} - {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level,
            entry.service,
            entry.message
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_valid_log_entry() {
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{"user_id":123}}"#;
        let parser = LogParser::new("test.log".to_string());
        let entry = parser.parse_line(log_line).unwrap();
        
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.service, "api");
        assert_eq!(entry.message, "Request processed");
        assert!(entry.metadata.is_some());
    }

    #[test]
    fn test_parse_invalid_json() {
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api""#;
        let parser = LogParser::new("test.log".to_string());
        let result = parser.parse_line(log_line);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::JsonError(_)));
    }

    #[test]
    fn test_level_filtering() {
        let temp_file = NamedTempFile::new().unwrap();
        let logs = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"DEBUG","service":"api","message":"Debug message"}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Info message"}
{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","service":"api","message":"Error message"}"#;
        
        write!(temp_file.as_file(), "{}", logs).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap().to_string())
            .with_min_level("INFO");
        
        let entries = parser.parse().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.level != "DEBUG"));
    }

    #[test]
    fn test_service_filtering() {
        let temp_file = NamedTempFile::new().unwrap();
        let logs = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"API log"}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"database","message":"DB log"}
{"timestamp":"2024-01-15T10:32:00Z","level":"INFO","service":"api","message":"Another API log"}"#;
        
        write!(temp_file.as_file(), "{}", logs).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap().to_string())
            .with_service_filter("api");
        
        let entries = parser.parse().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.service == "api"));
    }
}