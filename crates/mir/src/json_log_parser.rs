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
    min_level: Option<String>,
    field_filters: HashMap<String, Value>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            field_filters: HashMap::new(),
        }
    }

    pub fn set_min_level(&mut self, level: &str) -> &mut Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn add_field_filter(&mut self, key: &str, value: Value) -> &mut Self {
        self.field_filters.insert(key.to_string(), value);
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
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(min_level) = &self.min_level {
            if !self.is_level_allowed(&level, min_level) {
                return Err("Log level below minimum threshold".into());
            }
        }

        for (filter_key, filter_value) in &self.field_filters {
            if let Some(actual_value) = fields.get(filter_key) {
                if actual_value != filter_value {
                    return Err("Field filter mismatch".into());
                }
            } else {
                return Err("Required field not found".into());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let level_idx = levels.iter().position(|&l| l == level);
        let min_idx = levels.iter().position(|&l| l == min_level);
        
        match (level_idx, min_idx) {
            (Some(l), Some(m)) => l >= m,
            _ => false,
        }
    }
}

impl LogEntry {
    pub fn format(&self, template: &str) -> String {
        let mut result = template.to_string();
        result = result.replace("{timestamp}", &self.timestamp);
        result = result.replace("{level}", &self.level.to_uppercase());
        result = result.replace("{message}", &self.message);
        
        for (key, value) in &self.fields {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started","service":"api","version":"1.0.0"}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.fields.get("service").unwrap(), &json!("api"));
    }

    #[test]
    fn test_level_filtering() {
        let mut parser = LogParser::new();
        parser.set_min_level("warn");
        
        let info_log = r#"{"level":"info","message":"Test"}"#;
        let warn_log = r#"{"level":"warn","message":"Warning"}"#;
        
        assert!(parser.parse_line(info_log).is_err());
        assert!(parser.parse_line(warn_log).is_ok());
    }

    #[test]
    fn test_format_entry() {
        let entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "error".to_string(),
            message: "Database connection failed".to_string(),
            fields: {
                let mut map = HashMap::new();
                map.insert("retry_count".to_string(), json!(3));
                map.insert("timeout".to_string(), json!(30));
                map
            },
        };
        
        let formatted = entry.format("[{level}] {timestamp}: {message} (retries: {retry_count}, timeout: {timeout}s)");
        assert_eq!(formatted, "[ERROR] 2024-01-15T10:30:00Z: Database connection failed (retries: 3, timeout: 30s)");
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
    include_fields: Vec<String>,
    exclude_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            include_fields: Vec::new(),
            exclude_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_uppercase());
        self
    }

    pub fn include_field(&mut self, field: &str) -> &mut Self {
        self.include_fields.push(field.to_string());
        self
    }

    pub fn exclude_field(&mut self, field: &str) -> &mut Self {
        self.exclude_fields.push(field.to_string());
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
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO")
            .to_uppercase();

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filter mismatch".into());
            }
        }

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key == "timestamp" || key == "level" || key == "message" {
                    continue;
                }

                if !self.include_fields.is_empty() && !self.include_fields.contains(key) {
                    continue;
                }

                if self.exclude_fields.contains(key) {
                    continue;
                }

                fields.insert(key.clone(), value.clone());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!("[{}] {}: {}", entry.timestamp, entry.level, entry.message);
        
        if !entry.fields.is_empty() {
            output.push_str(" | ");
            let fields_str: Vec<String> = entry.fields.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&fields_str.join(", "));
        }

        output
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
    fn test_parse_valid_json() {
        let parser = LogParser::new();
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Failed to connect","service":"api","attempt":3}"#;
        
        let result = parser.parse_line(json_line);
        assert!(result.is_ok());
        
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Failed to connect");
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "api");
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("ERROR");
        
        let error_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Error occurred"}"#;
        let info_line = r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Processing complete"}"#;
        
        assert!(parser.parse_line(error_line).is_ok());
        assert!(parser.parse_line(info_line).is_err());
    }
}