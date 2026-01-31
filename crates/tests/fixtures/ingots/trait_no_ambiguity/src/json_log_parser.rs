use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    field_filters: HashMap<String, String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            field_filters: HashMap::new(),
        }
    }

    pub fn set_min_level(&mut self, level: &str) {
        self.min_level = Some(level.to_lowercase());
    }

    pub fn add_field_filter(&mut self, key: &str, value: &str) {
        self.field_filters.insert(key.to_string(), value.to_string());
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
            .unwrap_or("info")
            .to_string()
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            if !self.is_level_allowed(&level, min_level) {
                return Err("Log level below minimum threshold".into());
            }
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

        for (filter_key, filter_value) in &self.field_filters {
            if let Some(value) = fields.get(filter_key) {
                if value.as_str() != Some(filter_value) {
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

    fn is_level_allowed(&self, log_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let log_idx = levels.iter().position(|&l| l == log_level);
        let min_idx = levels.iter().position(|&l| l == min_level);
        
        match (log_idx, min_idx) {
            (Some(l), Some(m)) => l >= m,
            _ => true,
        }
    }
}

impl LogEntry {
    pub fn format(&self, show_fields: bool) -> String {
        let mut output = format!("[{}] {}: {}", self.timestamp, self.level.to_uppercase(), self.message);
        
        if show_fields && !self.fields.is_empty() {
            output.push_str(" | ");
            for (key, value) in &self.fields {
                output.push_str(&format!("{}={:?} ", key, value));
            }
        }
        
        output.trim().to_string()
    }
}