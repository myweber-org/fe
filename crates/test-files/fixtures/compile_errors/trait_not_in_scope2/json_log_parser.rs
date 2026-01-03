
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidLogFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        if line_content.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line_content)?;
        
        let entry = parse_json_log_entry(json_value)
            .map_err(|msg| ParseError::InvalidLogFormat(format!("Line {}: {}", line_num + 1, msg)))?;
        
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_json_log_entry(value: Value) -> Result<LogEntry, String> {
    let obj = value.as_object()
        .ok_or_else(|| "Log entry must be a JSON object".to_string())?;

    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid timestamp field".to_string())?
        .to_string();

    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid level field".to_string())?
        .to_string();

    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid message field".to_string())?
        .to_string();

    let metadata = obj.get("metadata")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log_entry() {
        let json_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "ERROR",
            "message": "Database connection failed",
            "metadata": {
                "service": "auth",
                "attempt": 3
            }
        });

        let entry = parse_json_log_entry(json_data).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.metadata["service"], "auth");
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Error 1".to_string(),
                metadata: Value::Null,
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Info 1".to_string(),
                metadata: Value::Null,
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Error 2".to_string(),
                metadata: Value::Null,
            },
        ];

        let error_logs = filter_logs_by_level(&entries, "ERROR");
        assert_eq!(error_logs.len(), 2);
        assert!(error_logs.iter().all(|e| e.level == "ERROR"));
    }
}