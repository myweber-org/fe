use serde_json::Value;
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

    pub fn set_level_filter(&mut self, level: &str) {
        self.filter_level = Some(level.to_lowercase());
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
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
            .unwrap_or("")
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filter mismatch".to_string());
            }
        }

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        for (key, value) in obj {
            if key != "timestamp" && key != "level" && key != "message" {
                fields.insert(key.clone(), value.clone());
            }
        }

        for required_field in &self.required_fields {
            if !fields.contains_key(required_field) {
                return Err(format!("Missing required field: {}", required_field));
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

    #[test]
    fn test_parse_valid_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","error_code":500,"service":"auth"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.level, "error");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("error_code").and_then(|v| v.as_i64()), Some(500));
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let error_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Error occurred"}"#;
        let info_log = r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Operation completed"}"#;
        
        assert!(parser.parse_line(error_log).is_ok());
        assert!(parser.parse_line(info_log).is_err());
    }
}
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Json(serde_json::Error),
    MalformedLine(String),
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

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {}", e),
            ParseError::Json(e) => write!(f, "JSON parsing error: {}", e),
            ParseError::MalformedLine(line) => write!(f, "Malformed log line: {}", line),
        }
    }
}

impl Error for ParseError {}

pub struct LogParser {
    reader: BufReader<File>,
    line_buffer: String,
}

impl LogParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(LogParser {
            reader,
            line_buffer: String::new(),
        })
    }

    pub fn parse_next(&mut self) -> Result<Option<LogEntry>, ParseError> {
        self.line_buffer.clear();
        
        match self.reader.read_line(&mut self.line_buffer) {
            Ok(0) => Ok(None),
            Ok(_) => {
                let trimmed = self.line_buffer.trim();
                if trimmed.is_empty() {
                    self.parse_next()
                } else {
                    match serde_json::from_str(trimmed) {
                        Ok(entry) => Ok(Some(entry)),
                        Err(e) => Err(ParseError::MalformedLine(trimmed.to_string())),
                    }
                }
            }
            Err(e) => Err(ParseError::Io(e)),
        }
    }

    pub fn collect_errors(&mut self) -> (Vec<LogEntry>, Vec<ParseError>) {
        let mut entries = Vec::new();
        let mut errors = Vec::new();

        while let Some(result) = self.parse_next().transpose() {
            match result {
                Ok(entry) => entries.push(entry),
                Err(err) => errors.push(err),
            }
        }

        (entries, errors)
    }
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

pub fn extract_service_names(entries: &[LogEntry]) -> Vec<&str> {
    let mut services: Vec<&str> = entries.iter().map(|e| e.service.as_str()).collect();
    services.sort_unstable();
    services.dedup();
    services
}