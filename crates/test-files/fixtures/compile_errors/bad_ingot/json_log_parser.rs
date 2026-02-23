use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format")]
    InvalidFormat,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    return Err(ParseError::InvalidFormat);
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        Ok(entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:00:00Z","level":"INFO","message":"System started"}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:01:00Z","level":"ERROR","message":"Disk full","disk_usage":95}}"#
        )
        .unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:00:00Z","level":"INFO","message":"Test"}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:01:00Z","level":"ERROR","message":"Error"}}"#
        )
        .unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error");
    }
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
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

    pub fn set_min_level(&mut self, level: &str) {
        let levels = ["trace", "debug", "info", "warn", "error"];
        if levels.contains(&level.to_lowercase().as_str()) {
            self.min_level = Some(level.to_lowercase());
        }
    }

    pub fn add_field_filter(&mut self, key: &str, value: Value) {
        self.field_filters.insert(key.to_string(), value);
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
        
        let timestamp_str = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or("Missing timestamp field")?;
        
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?
            .with_timezone(&Utc);

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            let level_order = |lvl: &str| match lvl {
                "trace" => 0,
                "debug" => 1,
                "info" => 2,
                "warn" => 3,
                "error" => 4,
                _ => 5,
            };

            if level_order(&level) < level_order(min_level) {
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
            if let Some(field_value) = fields.get(filter_key) {
                if field_value != filter_value {
                    return Err("Field filter mismatch".into());
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

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.fields.is_empty() {
            output.push_str(" | ");
            let fields_str: Vec<String> = entry.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&fields_str.join(", "));
        }

        output
    }
}

pub fn analyze_logs(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut stats = HashMap::new();
    
    for entry in entries {
        *stats.entry(entry.level.clone()).or_insert(0) += 1;
        
        for key in entry.fields.keys() {
            *stats.entry(format!("field_{}", key)).or_insert(0) += 1;
        }
    }
    
    stats
}use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct LogParser {
    min_level: String,
    service_filter: Option<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, Box<dyn Error>> {
        let entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.matches_level(&entry.level) {
            return Ok(None);
        }

        if let Some(ref service) = self.service_filter {
            if !entry.service.eq_ignore_ascii_case(service) {
                return Ok(None);
            }
        }

        Ok(Some(entry))
    }

    fn matches_level(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let min_idx = level_order.iter().position(|&l| l == self.min_level);
        let entry_idx = level_order.iter().position(|&l| l == level.to_lowercase().as_str());
        
        match (min_idx, entry_idx) {
            (Some(min), Some(entry)) => entry >= min,
            _ => false,
        }
    }
}

pub fn count_errors(entries: &[LogEntry]) -> usize {
    entries.iter()
        .filter(|e| e.level.eq_ignore_ascii_case("error"))
        .count()
}

pub fn extract_messages(entries: &[LogEntry]) -> Vec<String> {
    entries.iter()
        .map(|e| format!("[{}] {}", e.level.to_uppercase(), e.message))
        .collect()
}