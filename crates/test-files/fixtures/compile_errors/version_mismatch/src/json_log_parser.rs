use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error at line {line}: {source}")]
    JsonParse {
        line: usize,
        source: serde_json::Error,
    },
    #[error("Missing required field '{field}' at line {line}")]
    MissingField { line: usize, field: String },
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file(path: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let line_number = line_num + 1;

        if line.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line)
            .map_err(|e| LogParseError::JsonParse {
                line: line_number,
                source: e,
            })?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "timestamp".to_string(),
            })?
            .to_string();

        let level = json_value["level"]
            .as_str()
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "level".to_string(),
            })?
            .to_string();

        let message = json_value["message"]
            .as_str()
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "message".to_string(),
            })?
            .to_string();

        let metadata = json_value.get("metadata").cloned().unwrap_or(Value::Null);

        entries.push(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        });
    }

    Ok(entries)
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","metadata":{"attempt":3}}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Service started","metadata":null}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let result = parse_json_log_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "ERROR");
        assert_eq!(entries[1].message, "Service started");
    }

    #[test]
    fn test_filter_error_logs() {
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
        assert!(error_logs.iter().all(|log| log.level == "ERROR"));
    }
}