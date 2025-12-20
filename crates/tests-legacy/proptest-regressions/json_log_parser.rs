
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
    entries: Vec<LogEntry>,
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
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let level = json_value["level"]
            .as_str()
            .unwrap_or("info")
            .to_string();

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

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn search_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let parser = LogParser::new();
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","error_code":500}"#;
        
        let entry = parser.parse_line(json_line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("error_code").unwrap().as_i64().unwrap(), 500);
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        parser.entries.push(LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
            fields: HashMap::new(),
        });
        parser.entries.push(LogEntry {
            timestamp: "2024-01-15T10:31:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Test info".to_string(),
            fields: HashMap::new(),
        });

        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}