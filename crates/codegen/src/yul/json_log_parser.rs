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

    pub fn require_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
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

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below minimum threshold".into());
        }

        let message = json_value.get("message")
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

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        let min_index = level_order.iter().position(|&l| l == self.min_level).unwrap_or(0);
        let current_index = level_order.iter().position(|&l| l == level).unwrap_or(0);
        current_index >= min_index
    }
}

pub fn filter_entries_by_field(entries: &[LogEntry], field: &str, value: &Value) -> Vec<LogEntry> {
    entries.iter()
        .filter(|entry| entry.fields.get(field) == Some(value))
        .cloned()
        .collect()
}

pub fn extract_field_values(entries: &[LogEntry], field: &str) -> Vec<Value> {
    entries.iter()
        .filter_map(|entry| entry.fields.get(field).cloned())
        .collect()
}