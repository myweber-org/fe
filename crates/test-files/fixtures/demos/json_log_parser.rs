use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level = json_value["level"]
            .as_str()
            .unwrap_or("info")
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
                    if self.required_fields.is_empty() || self.required_fields.contains(key) {
                        fields.insert(key.clone(), value.clone());
                    }
                }
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
            .filter_map(|entry| entry.fields.get(field_name).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_without_filter() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","error_code":500,"service":"auth"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.level, "error");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("error_code").unwrap().as_i64(), Some(500));
    }

    #[test]
    fn test_parser_with_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let error_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed"}"#;
        let info_line = r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Service started"}"#;
        
        assert!(parser.parse_line(error_line).is_ok());
        assert!(parser.parse_line(info_line).is_err());
    }

    #[test]
    fn test_field_extraction() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Failed","user_id":12345,"ip":"192.168.1.1"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        let user_ids = parser.extract_field_values(&[entry], "user_id");
        
        assert_eq!(user_ids.len(), 1);
        assert_eq!(user_ids[0].as_i64(), Some(12345));
    }
}