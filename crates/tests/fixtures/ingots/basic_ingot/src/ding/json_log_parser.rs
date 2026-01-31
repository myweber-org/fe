use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    error: Option<String>,
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_errors(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = parse_log_file("application.log")?;
    let error_entries = filter_errors(entries);

    println!("Found {} error entries:", error_entries.len());
    for entry in error_entries {
        println!(
            "[{}] {} - {}",
            entry.timestamp,
            entry.level,
            entry.error.unwrap_or(entry.message)
        );
    }

    Ok(())
}use serde_json::Value;
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
    filter_level: Option<String>,
    extract_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            extract_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
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
            level: json_value.get("level").and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
            message: json_value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fields: HashMap::new(),
        };

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    entry.fields.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(filter_level) = &self.filter_level {
            if let Some(entry_level) = &entry.level {
                if entry_level != filter_level {
                    return Err("Level filter mismatch".into());
                }
            }
        }

        Ok(entry)
    }

    pub fn extract_specific_fields(&self, entry: &LogEntry) -> HashMap<String, Value> {
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

    #[test]
    fn test_parse_valid_json_log() {
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Failed to connect","service":"api","error_code":500}"#;
        let parser = LogParser::new();
        let entry = parser.parse_line(log_line).unwrap();
        
        assert_eq!(entry.timestamp.unwrap(), "2024-01-15T10:30:00Z");
        assert_eq!(entry.level.unwrap(), "error");
        assert_eq!(entry.message.unwrap(), "Failed to connect");
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "api");
        assert_eq!(entry.fields.get("error_code").unwrap().as_i64().unwrap(), 500);
    }

    #[test]
    fn test_level_filter() {
        let log_line = r#"{"level":"error","message":"Test"}"#;
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let entry = parser.parse_line(log_line);
        assert!(entry.is_ok());
    }
}