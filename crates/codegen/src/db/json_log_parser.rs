
use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(JsonError),
    InvalidLogFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<JsonError> for ParseError {
    fn from(err: JsonError) -> Self {
        ParseError::JsonError(err)
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        JsonLogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }
            
            let json_value: Value = serde_json::from_str(&line_content)?;
            
            let entry = self.parse_json_to_entry(json_value, line_num + 1)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    fn parse_json_to_entry(&self, json: Value, line_num: usize) -> Result<LogEntry, ParseError> {
        if !json.is_object() {
            return Err(ParseError::InvalidLogFormat(
                format!("Line {}: Expected JSON object", line_num)
            ));
        }
        
        let obj = json.as_object().unwrap();
        
        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::InvalidLogFormat(
                format!("Line {}: Missing or invalid timestamp", line_num)
            ))?;
        
        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::InvalidLogFormat(
                format!("Line {}: Missing or invalid level", line_num)
            ))?;
        
        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::InvalidLogFormat(
                format!("Line {}: Missing or invalid message", line_num)
            ))?;
        
        let mut metadata = json.clone();
        if let Some(obj) = metadata.as_object_mut() {
            obj.remove("timestamp");
            obj.remove("level");
            obj.remove("message");
        }
        
        Ok(LogEntry {
            timestamp: timestamp.to_string(),
            level: level.to_string(),
            message: message.to_string(),
            metadata,
        })
    }
    
    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        
        Ok(filtered)
    }
    
    pub fn count_entries_by_level(&self) -> Result<std::collections::HashMap<String, usize>, ParseError> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_parse_valid_logs() {
        let log_content = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "INFO", "message": "Application started", "user_id": 123}
{"timestamp": "2023-10-01T12:01:00Z", "level": "ERROR", "message": "Database connection failed", "error_code": 500}
{"timestamp": "2023-10-01T12:02:00Z", "level": "WARN", "message": "High memory usage", "memory_mb": 2048}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();
        
        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
        assert_eq!(entries[2].level, "WARN");
    }
    
    #[test]
    fn test_filter_by_level() {
        let log_content = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "INFO", "message": "Test 1"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "ERROR", "message": "Test 2"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "INFO", "message": "Test 3"}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();
        
        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let info_entries = parser.filter_by_level("INFO").unwrap();
        
        assert_eq!(info_entries.len(), 2);
        assert!(info_entries.iter().all(|e| e.level == "INFO"));
    }
    
    #[test]
    fn test_count_entries_by_level() {
        let log_content = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "INFO", "message": "Test 1"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "ERROR", "message": "Test 2"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "INFO", "message": "Test 3"}
{"timestamp": "2023-10-01T12:03:00Z", "level": "WARN", "message": "Test 4"}
{"timestamp": "2023-10-01T12:04:00Z", "level": "INFO", "message": "Test 5"}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();
        
        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let counts = parser.count_entries_by_level().unwrap();
        
        assert_eq!(counts.get("INFO"), Some(&3));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}