use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            required_fields: Vec::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
        
        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below threshold".into());
        }

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                fields.insert(key.clone(), value.clone());
            }
        }

        let timestamp = fields.get("timestamp")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let message = fields.get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());

        if !self.required_fields.is_empty() {
            for field in &self.required_fields {
                if !fields.contains_key(field) {
                    return Err(format!("Missing required field: {}", field).into());
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

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        
        let log_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(level_order.len());

        log_index >= min_index
    }
}

pub fn extract_field_values(entries: &[LogEntry], field_name: &str) -> Vec<Value> {
    entries.iter()
        .filter_map(|entry| entry.fields.get(field_name))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("info");
        assert!(parser.is_level_allowed("info"));
        assert!(parser.is_level_allowed("error"));
        assert!(!parser.is_level_allowed("debug"));
    }

    #[test]
    fn test_parse_valid_json() {
        let parser = LogParser::new("debug");
        let log_line = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"Test message","user_id":123}"#;
        
        let result = parser.parse_line(log_line);
        assert!(result.is_ok());
        
        let entry = result.unwrap();
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.fields.get("user_id").unwrap().as_i64().unwrap(), 123);
    }
}