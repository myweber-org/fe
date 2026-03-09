
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line_content)?;
            
            if !json_value.is_object() {
                return Err(LogParseError::MissingField(
                    format!("Line {}: Expected JSON object", line_num + 1)
                ));
            }

            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let logs = self.parse()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    pub fn extract_timestamps(&self) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse()?;
        let timestamps: Vec<String> = logs
            .iter()
            .filter_map(|log| {
                log.get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        if timestamps.is_empty() {
            return Err(LogParseError::MissingField(
                "No timestamps found in logs".to_string()
            ));
        }

        Ok(timestamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_logs() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:32:00Z", "level": "WARN", "message": "High memory usage"}}"#).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_logs() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["level"], "ERROR");
    }

    #[test]
    fn test_extract_timestamps() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let timestamps = parser.extract_timestamps().unwrap();
        assert_eq!(timestamps.len(), 3);
        assert!(timestamps[0].contains("2024-01-15"));
    }
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filter_level: Option<String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            required_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn add_required_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, String> {
        let json_value: Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let obj = json_value.as_object()
            .ok_or("Log entry must be a JSON object")?;

        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or("Missing timestamp field")?
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or("Missing level field")?
            .to_string()
            .to_lowercase();

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filter mismatch".to_string());
            }
        }

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .ok_or("Missing message field")?
            .to_string();

        let mut fields = HashMap::new();
        for (key, value) in obj {
            if !["timestamp", "level", "message"].contains(&key.as_str()) {
                fields.insert(key.clone(), value.clone());
            }
        }

        for required in &self.required_fields {
            if !fields.contains_key(required) && !obj.contains_key(required) {
                return Err(format!("Missing required field: {}", required));
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries.iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Database connection failed","error_code":500,"service":"auth"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.level, "error");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("error_code").unwrap(), &json!(500));
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("info");
        
        let info_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started"}"#;
        let error_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Failed"}"#;
        
        assert!(parser.parse_line(info_log).is_ok());
        assert!(parser.parse_line(error_log).is_err());
    }
}