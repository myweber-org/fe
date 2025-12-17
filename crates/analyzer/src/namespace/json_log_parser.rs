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
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let obj = json_value.as_object()
            .ok_or("Log entry must be a JSON object")?;

        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or("Missing timestamp field")?
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or("Missing level field")?
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
            if !fields.contains_key(required) {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut parser = LogParser::new();
        parser.set_level_filter("info");

        let log_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INFO",
            "message": "System started",
            "user_id": 12345,
            "service": "auth"
        });

        let entry = parser.parse_line(&log_data.to_string()).unwrap();
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "System started");
        assert_eq!(entry.fields.get("user_id"), Some(&json!(12345)));
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");

        let log_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "WARN",
            "message": "Disk space low"
        });

        let result = parser.parse_line(&log_data.to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_required_fields() {
        let mut parser = LogParser::new();
        parser.add_required_field("request_id");

        let log_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INFO",
            "message": "Request processed"
        });

        let result = parser.parse_line(&log_data.to_string());
        assert!(result.is_err());

        let log_data_with_field = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "INFO",
            "message": "Request processed",
            "request_id": "abc-123"
        });

        let entry = parser.parse_line(&log_data_with_field.to_string()).unwrap();
        assert_eq!(entry.fields.get("request_id"), Some(&json!("abc-123")));
    }

    #[test]
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let logs = vec![
            json!({
                "timestamp": "2024-01-15T10:30:00Z",
                "level": "INFO",
                "message": "Startup complete",
                "version": "1.0.0"
            }),
            json!({
                "timestamp": "2024-01-15T10:31:00Z",
                "level": "ERROR",
                "message": "Database connection failed",
                "error_code": 500
            }),
        ];

        for log in logs {
            writeln!(temp_file, "{}", log.to_string()).unwrap();
        }

        let parser = LogParser::new();
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].level, "error");
    }
}