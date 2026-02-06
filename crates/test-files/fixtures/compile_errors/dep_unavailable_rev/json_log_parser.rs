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
    pub entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line)?;
        }

        Ok(())
    }

    pub fn parse_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = json_value
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO")
            .to_string()
            .to_uppercase();

        let message = json_value
            .get("message")
            .and_then(|v| v.as_str())
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

        self.entries.push(LogEntry {
            timestamp,
            level,
            message,
            fields,
        });

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        let target_level = level.to_uppercase();
        self.entries
            .iter()
            .filter(|entry| entry.level == target_level)
            .collect()
    }

    pub fn extract_field_values(&self, field_name: &str) -> Vec<&Value> {
        self.entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_timeline(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.timestamp.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let mut parser = LogParser::new();
        let line = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"error","message":"Something went wrong","user_id":123,"ip":"192.168.1.1"}"#;
        
        assert!(parser.parse_line(line).is_ok());
        assert_eq!(parser.entries.len(), 1);
        
        let entry = &parser.entries[0];
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Something went wrong");
        assert_eq!(entry.fields.len(), 2);
        assert!(entry.fields.contains_key("user_id"));
        assert!(entry.fields.contains_key("ip"));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        parser.parse_line(r#"{"level":"error","message":"Error 1"}"#).unwrap();
        parser.parse_line(r#"{"level":"info","message":"Info 1"}"#).unwrap();
        parser.parse_line(r#"{"level":"error","message":"Error 2"}"#).unwrap();
        
        let errors = parser.filter_by_level("error");
        assert_eq!(errors.len(), 2);
        
        let infos = parser.filter_by_level("info");
        assert_eq!(infos.len(), 1);
    }

    #[test]
    fn test_count_by_level() {
        let mut parser = LogParser::new();
        parser.parse_line(r#"{"level":"error","message":"Error 1"}"#).unwrap();
        parser.parse_line(r#"{"level":"info","message":"Info 1"}"#).unwrap();
        parser.parse_line(r#"{"level":"error","message":"Error 2"}"#).unwrap();
        parser.parse_line(r#"{"level":"warning","message":"Warning 1"}"#).unwrap();
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("ERROR"), Some(&2));
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("WARNING"), Some(&1));
    }
}