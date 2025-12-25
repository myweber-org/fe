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

    pub fn with_level_filter(mut self, level: &str) -> Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn with_required_fields(mut self, fields: &[&str]) -> Self {
        self.required_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level = json_value["level"]
            .as_str()
            .unwrap_or("INFO")
            .to_string()
            .to_lowercase();

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filter mismatch".into());
            }
        }

        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        for field in &self.required_fields {
            if !fields.contains_key(field) {
                return Err(format!("Missing required field: {}", field).into());
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
        entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .cloned()
            .collect()
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","error_code":500,"service":"auth"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "error");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("error_code").unwrap().as_i64().unwrap(), 500);
    }

    #[test]
    fn test_level_filter() {
        let parser = LogParser::new().with_level_filter("error");
        let error_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Error occurred"}"#;
        let info_log = r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Operation completed"}"#;
        
        assert!(parser.parse_line(error_log).is_ok());
        assert!(parser.parse_line(info_log).is_err());
    }

    #[test]
    fn test_required_fields() {
        let parser = LogParser::new().with_required_fields(&["user_id", "action"]);
        let valid_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"User login","user_id":123,"action":"login"}"#;
        let invalid_log = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started"}"#;
        
        assert!(parser.parse_line(valid_log).is_ok());
        assert!(parser.parse_line(invalid_log).is_err());
    }
}