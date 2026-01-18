use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filters: Vec<Filter>,
    extract_fields: Vec<String>,
}

#[derive(Debug)]
pub enum Filter {
    Level(String),
    FieldEquals(String, Value),
    FieldContains(String, String),
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            extract_fields: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    pub fn add_extract_field(&mut self, field: &str) -> &mut Self {
        self.extract_fields.push(field.to_string());
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
        
        let mut entry = LogEntry {
            timestamp: json_value.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()),
            level: json_value.get("level").and_then(|v| v.as_str()).map(|s| s.to_string()),
            message: json_value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fields: HashMap::new(),
        };

        if let Value::Object(map) = json_value {
            for (key, value) in map {
                if !matches!(key.as_str(), "timestamp" | "level" | "message") {
                    entry.fields.insert(key, value.clone());
                }
            }
        }

        if !self.filters.is_empty() && !self.passes_filters(&entry) {
            return Err("Entry does not pass filters".into());
        }

        Ok(entry)
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        for filter in &self.filters {
            match filter {
                Filter::Level(level) => {
                    if let Some(entry_level) = &entry.level {
                        if entry_level != level {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                Filter::FieldEquals(field, value) => {
                    if let Some(field_value) = entry.fields.get(field) {
                        if field_value != value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                Filter::FieldContains(field, substring) => {
                    if let Some(field_value) = entry.fields.get(field) {
                        if let Some(field_str) = field_value.as_str() {
                            if !field_str.contains(substring) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn extract_selected_fields(&self, entry: &LogEntry) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        
        for field in &self.extract_fields {
            if let Some(value) = entry.fields.get(field) {
                result.insert(field.clone(), value.clone());
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_json() {
        let parser = LogParser::new();
        let line = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "INFO", "message": "Test message", "user_id": 123}"#;
        
        let entry = parser.parse_line(line).unwrap();
        assert_eq!(entry.timestamp, Some("2023-10-01T12:00:00Z".to_string()));
        assert_eq!(entry.level, Some("INFO".to_string()));
        assert_eq!(entry.message, Some("Test message".to_string()));
        assert_eq!(entry.fields.get("user_id"), Some(&json!(123)));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        parser.add_filter(Filter::Level("ERROR".to_string()));
        
        let error_line = r#"{"level": "ERROR", "message": "Error occurred"}"#;
        let info_line = r#"{"level": "INFO", "message": "Info message"}"#;
        
        assert!(parser.parse_line(error_line).is_ok());
        assert!(parser.parse_line(info_line).is_err());
    }

    #[test]
    fn test_extract_fields() {
        let mut parser = LogParser::new();
        parser.add_extract_field("user_id");
        parser.add_extract_field("session_id");
        
        let line = r#"{"level": "INFO", "user_id": 456, "session_id": "abc123", "extra": "data"}"#;
        let entry = parser.parse_line(line).unwrap();
        let extracted = parser.extract_selected_fields(&entry);
        
        assert_eq!(extracted.get("user_id"), Some(&json!(456)));
        assert_eq!(extracted.get("session_id"), Some(&json!("abc123")));
        assert_eq!(extracted.get("extra"), None);
    }
}