use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum LogParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidLogFormat(String),
}

impl From<std::io::Error> for LogParseError {
    fn from(err: std::io::Error) -> Self {
        LogParseError::IoError(err)
    }
}

impl From<serde_json::Error> for LogParseError {
    fn from(err: serde_json::Error) -> Self {
        LogParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: Value,
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        if line_content.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line_content)?;
        
        let entry = parse_json_to_logentry(json_value)
            .map_err(|e| LogParseError::InvalidLogFormat(
                format!("Line {}: {}", line_num + 1, e)
            ))?;
        
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_json_to_logentry(value: Value) -> Result<LogEntry, String> {
    let obj = value.as_object()
        .ok_or_else(|| "Log entry must be a JSON object".to_string())?;

    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid timestamp field".to_string())?
        .to_string();

    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid level field".to_string())?
        .to_uppercase();

    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing or invalid message field".to_string())?
        .to_string();

    let mut fields = value.clone();
    if let Some(obj) = fields.as_object_mut() {
        obj.remove("timestamp");
        obj.remove("level");
        obj.remove("message");
    }

    Ok(LogEntry {
        timestamp,
        level,
        message,
        fields,
    })
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    let target_level = level.to_uppercase();
    entries.iter()
        .filter(|entry| entry.level == target_level)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"info","message":"System started","user":"admin"}"#;
        writeln!(temp_file, "{}", log_data).unwrap();

        let entries = parse_log_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[0].message, "System started");
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2023-10-01T12:00:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Failed to connect".to_string(),
                fields: json!({}),
            },
            LogEntry {
                timestamp: "2023-10-01T12:01:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Connection established".to_string(),
                fields: json!({}),
            },
        ];

        let errors = filter_logs_by_level(&entries, "error");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Failed to connect");
    }
}use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Json(serde_json::Error),
    MalformedLine(usize, String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::Json(err)
    }
}

pub struct LogParser {
    path: String,
}

impl LogParser {
    pub fn new(path: impl Into<String>) -> Self {
        LogParser { path: path.into() }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => return Err(ParseError::MalformedLine(line_num + 1, e.to_string())),
            }
        }
        
        Ok(entries)
    }
    
    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        Ok(entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{{"user_id":123}}}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Connection failed","metadata":{{"retry_count":3}}}}"#).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].service, "db");
    }
    
    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:32:00Z","level":"INFO","service":"cache","message":"Test"}}"#).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].service, "db");
    }
}