use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    ValidationError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::JsonError(e) => write!(f, "JSON parsing error: {}", e),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ParseError {}

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

impl LogEntry {
    pub fn validate(&self) -> Result<(), ParseError> {
        if self.timestamp.is_empty() {
            return Err(ParseError::ValidationError(
                "Timestamp cannot be empty".to_string(),
            ));
        }
        if !["ERROR", "WARN", "INFO", "DEBUG"].contains(&self.level.as_str()) {
            return Err(ParseError::ValidationError(format!(
                "Invalid log level: {}",
                self.level
            )));
        }
        if self.service.is_empty() {
            return Err(ParseError::ValidationError(
                "Service name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn is_error(&self) -> bool {
        self.level == "ERROR"
    }

    pub fn service_name(&self) -> &str {
        &self.service
    }
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            entry.validate()?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        Ok(entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect())
    }

    pub fn count_errors_by_service(&self) -> Result<Vec<(String, usize)>, ParseError> {
        let entries = self.parse()?;
        let mut service_counts = std::collections::HashMap::new();

        for entry in entries {
            if entry.is_error() {
                *service_counts.entry(entry.service_name().to_string()).or_insert(0) += 1;
            }
        }

        let mut result: Vec<(String, usize)> = service_counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_log_entry() {
        let json = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "ERROR",
            "service": "api-gateway",
            "message": "Connection timeout",
            "metadata": {"duration_ms": 5000}
        }"#;

        let entry: Result<LogEntry, _> = serde_json::from_str(json);
        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.service, "api-gateway");
        assert!(entry.is_error());
    }

    #[test]
    fn test_invalid_log_level() {
        let json = r#"{
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "CRITICAL",
            "service": "api-gateway",
            "message": "Test message"
        }"#;

        let entry: LogEntry = serde_json::from_str(json).unwrap();
        let result = entry.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_with_temp_file() -> Result<(), Box<dyn Error>> {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "service": "api", "message": "Error 1"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "service": "db", "message": "Query executed"}
{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "service": "api", "message": "Error 2"}"#;

        let temp_file = NamedTempFile::new()?;
        std::fs::write(temp_file.path(), log_data)?;

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse()?;
        assert_eq!(entries.len(), 3);

        let error_entries = parser.filter_by_level("ERROR")?;
        assert_eq!(error_entries.len(), 2);

        let error_counts = parser.count_errors_by_service()?;
        assert_eq!(error_counts.len(), 1);
        assert_eq!(error_counts[0], ("api".to_string(), 2));

        Ok(())
    }
}